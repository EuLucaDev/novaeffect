//! Colaboração em tempo real (Nova Live) no motor do Nova Effect (um projeto por sessão).
//!
//! - Comandos do diário viram [`LiveChange::Op`] com o estado de edição de quem editou
//!   (composição ativa, camadas selecionadas, tempo), para o colega rodar o mesmo comando no mesmo lugar.
//! - Qualquer outra mudança do projeto (desfazer, refazer, gestos) vira [`LiveChange::Replace`].
//! - O estado de edição de cada pessoa continua dela.
//! - Mídias (vídeo, áudio, imagem): o projeto guarda o caminho de quem importou. Quem não tem o
//!   arquivo recebe uma cópia pela sessão ([`Session::live_media_ready`]); o projeto continua
//!   igual em todos e cada um lê a cópia local no lugar do caminho.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use novaeffect_project::{ItemKind, Project};
use serde_json::{Value, json};

use crate::{EditorState, EngineError, Result, Services, Session};

#[derive(Clone, Debug, PartialEq)]
pub enum LiveChange {
    Op { cmd: String, params: Value, ctx: Value },
    Replace,
}

#[derive(Default)]
pub struct LiveState {
    pub on: bool,
    pub outbox: Vec<LiveChange>,
    pub(crate) applying: bool,
    /// O projeto já informado à sessão (só muda de objeto quando o conteúdo muda).
    pub(crate) seen: Option<Arc<Project>>,
    /// Comandos rodando agora (comandos que chamam comandos): só o de fora é replicado.
    pub(crate) depth: u32,
    /// Mídias recebidas: caminho no projeto → cópia local (continua valendo depois da sessão).
    pub aliases: MediaAliases,
}

/// Caminho de mídia do projeto → arquivo local recebido numa sessão.
#[derive(Clone, Debug, Default)]
pub struct MediaAliases(Arc<RwLock<HashMap<String, String>>>);

impl MediaAliases {
    pub fn get(&self, path: &str) -> Option<String> {
        self.0.read().unwrap_or_else(|e| e.into_inner()).get(path).cloned()
    }
    pub fn set(&self, path: &str, local: &str) {
        self.0.write().unwrap_or_else(|e| e.into_inner()).insert(path.to_string(), local.to_string());
    }
}

/// Os serviços do app, lendo a cópia recebida no lugar do caminho do projeto.
struct AliasServices {
    inner: Arc<dyn Services>,
    aliases: MediaAliases,
}

impl AliasServices {
    fn map(&self, path: &str) -> String {
        self.aliases.get(path).unwrap_or_else(|| path.to_string())
    }
}

impl Services for AliasServices {
    fn read_file(&self, path: &str) -> std::io::Result<Vec<u8>> {
        self.inner.read_file(&self.map(path))
    }
    fn write_file(&self, path: &str, data: &[u8]) -> std::io::Result<()> {
        self.inner.write_file(path, data)
    }
    fn exists(&self, path: &str) -> bool {
        self.inner.exists(&self.map(path))
    }
    fn stamp(&self, path: &str) -> Option<(u64, u128)> {
        self.inner.stamp(&self.map(path))
    }
    fn store_file(&self, path: &str, data: &[u8]) -> std::io::Result<()> {
        self.inner.store_file(path, data)
    }
    fn list_dir(&self, dir: &str) -> Vec<String> {
        self.inner.list_dir(dir)
    }
    fn is_dir(&self, path: &str) -> bool {
        self.inner.is_dir(path)
    }
}

fn err(e: impl std::fmt::Display) -> EngineError {
    EngineError::Other(format!("colaboração: {e}"))
}

/// Impressão digital estável (FNV-1a 64).
pub fn fnv64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

// Only audited document edits may execute on a peer. Other local edits travel as data.
fn remote_command_allowed(cmd: &str) -> bool {
    matches!(cmd, "layer.newSolid")
}

impl Session {
    pub fn live_start(&mut self) -> Result<()> {
        let aliases = self.live.aliases.clone();
        self.services = Arc::new(AliasServices { inner: self.services.clone(), aliases: aliases.clone() });
        self.live = LiveState { on: true, seen: Some(self.project.clone()), aliases, ..LiveState::default() };
        Ok(())
    }

    pub fn live_stop(&mut self) {
        let aliases = self.live.aliases.clone();
        self.live = LiveState { aliases, ..LiveState::default() };
    }

    /// Arquivos que o projeto usa (footage e proxies), pelo caminho guardado no projeto.
    pub fn live_media(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for item in self.project.items.values() {
            let own = match &item.kind {
                ItemKind::Footage(f) => Some(&f.path),
                _ => None,
            };
            for path in own.into_iter().chain(item.proxy.as_ref().map(|p| &p.footage.path)) {
                if !path.is_empty() && !out.contains(path) {
                    out.push(path.clone());
                }
            }
        }
        out
    }

    /// A mídia existe aqui (no caminho ou como cópia recebida)?
    pub fn live_media_available(&self, path: &str) -> bool {
        self.services.exists(path)
    }

    /// O arquivo local da mídia, para enviar a um colega (só se estiver aqui).
    pub fn live_media_file(&self, path: &str) -> Option<PathBuf> {
        let local = self.live.aliases.get(path).unwrap_or_else(|| path.to_string());
        Path::new(&local).is_file().then(|| PathBuf::from(local))
    }

    /// A mídia chegou: o projeto passa a ler `local` no lugar de `path`.
    pub fn live_media_ready(&mut self, path: &str, local: &Path) {
        let local = local.to_string_lossy();
        self.live.aliases.set(path, &local);
        self.footage.set_alias(path, &local);
        self.layer_cache.clear();
    }

    pub fn live_active(&self) -> bool {
        self.live.on
    }

    pub fn live_take(&mut self) -> Vec<LiveChange> {
        if self.live.on && !self.live.seen.as_ref().is_some_and(|s| Arc::ptr_eq(s, &self.project)) {
            self.live.seen = Some(self.project.clone());
            self.live.outbox.push(LiveChange::Replace);
        }
        std::mem::take(&mut self.live.outbox)
    }

    pub fn live_busy(&self) -> bool {
        false
    }

    pub fn live_snapshot(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&*self.project).map_err(err)
    }

    pub fn live_digest(&self) -> Result<u64> {
        Ok(fnv64(&self.live_snapshot()?))
    }

    pub fn live_load(&mut self, bytes: &[u8]) -> Result<()> {
        let p: Project = serde_json::from_slice(bytes).map_err(err)?;
        self.project = Arc::new(p);
        self.revision += 1;
        self.live.seen = Some(self.project.clone());
        // O desfazer guarda fotos do projeto: depois de uma alteração de colega, desfazer o passo
        // anterior apagaria o trabalho dele. O histórico local recomeça daqui.
        self.history.undo.clear();
        self.history.redo.clear();

        Ok(())
    }

    /// Roda o comando de um colega com o estado de edição dele; desfazer e estado locais ficam.
    pub fn live_apply(&mut self, cmd: &str, params: &Value, ctx: &Value) -> Result<()> {
        if !remote_command_allowed(cmd) {
            return Err(err("comando não permitido na colaboração"));
        }
        let local = self.state.clone();
        if let Some(state) = ctx.get("state").and_then(|s| serde_json::from_value::<EditorState>(s.clone()).ok()) {
            self.state = state;
        }
        self.live.applying = true;
        let r = self.execute(cmd, params.clone());
        self.live.applying = false;
        self.history.undo.clear();
        self.history.redo.clear();
        self.state = local;
        self.live.seen = Some(self.project.clone());

        r.map(|_| ())
    }

    /// Chamado por `execute` depois de cada comando de fora.
    pub(crate) fn live_record(&mut self, id: &str, params: &Value, replicable: bool, ctx: Value) {
        if self.live.applying || !self.live.on || self.live.seen.as_ref().is_some_and(|s| Arc::ptr_eq(s, &self.project)) {
            return;
        }
        if replicable && remote_command_allowed(id) && !matches!(id, "edit.undo" | "edit.redo") {
            self.live.outbox.push(LiveChange::Op { cmd: id.to_string(), params: params.clone(), ctx });
            self.live.seen = Some(self.project.clone());
        }
    }
}

pub(crate) fn context(state: &EditorState) -> Value {
    json!({ "state": state })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    fn session() -> Session {
        let mut s = Session::default();
        s.execute("comp.new", json!({"width": 400, "height": 300, "duration": 2})).unwrap();
        s
    }

    #[test]
    fn remote_commands_cannot_open_export_or_execute_code() {
        let (_, mut b) = pair();
        let before = b.live_digest().unwrap();
        for command in ["file.open", "file.saveAs", "file.export", "session.open", "script.run", "app.quit", "lisp.eval"] {
            assert!(b.live_apply(command, &serde_json::json!({"path": "/tmp/not-authorized"}), &serde_json::json!({})).is_err());
            assert_eq!(before, b.live_digest().unwrap());
        }
    }

    fn pair() -> (Session, Session) {
        let mut a = session();
        a.live_start().unwrap();
        let mut b = session();
        b.live_start().unwrap();
        let snap = a.live_snapshot().unwrap();
        b.live_load(&snap).unwrap();
        (a, b)
    }

    fn sync(a: &mut Session, b: &mut Session) {
        for change in a.live_take() {
            match change {
                LiveChange::Op { cmd, params, ctx } => b.live_apply(&cmd, &params, &ctx).unwrap(),
                LiveChange::Replace => {
                    let snap = a.live_snapshot().unwrap();
                    b.live_load(&snap).unwrap();
                }
            }
        }
        assert!(b.live_take().is_empty(), "o que veio de A não volta para A");
    }

    #[test]
    fn edits_replicate_and_undo_travels_as_a_project() {
        let (mut a, mut b) = pair();
        let b_undo = b.history.undo.len();
        a.execute("layer.newSolid", json!({"name": "Fundo", "width": 400, "height": 300})).unwrap();
        a.execute("layer.newSolid", json!({"name": "Logo", "width": 100, "height": 100})).unwrap();
        sync(&mut a, &mut b);
        assert_eq!(a.live_digest().unwrap(), b.live_digest().unwrap());
        assert!(b.history.undo.is_empty(), "o desfazer de B não ganha os passos de A (e recomeça)");
        let _ = b_undo;

        a.execute("edit.undo", json!({})).unwrap();
        sync(&mut a, &mut b);
        assert_eq!(a.live_digest().unwrap(), b.live_digest().unwrap());

        b.execute("layer.newSolid", json!({"name": "Texto", "width": 200, "height": 50})).unwrap();
        sync(&mut b, &mut a);
        assert_eq!(a.live_digest().unwrap(), b.live_digest().unwrap());
    }
}
