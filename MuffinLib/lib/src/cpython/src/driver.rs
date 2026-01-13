//! Muffin CPython – Driver (MAX)
//!
//! Driver d’exécution Python pour Muffin.
//!
//! Responsabilités :
//! - détecter l’environnement Python
//! - valider la compatibilité backend / action
//! - appliquer l’environnement d’exécution
//! - appeler le runner Muffin
//!
//! Hors périmètre :
//! - graphe
//! - planification
//! - cache CAS
//! - résolution dépendances

use std::path::PathBuf;

use super::error::MuffinError;
use super::runner::process::CommandRunner;

use super::args::{PyArgs, PyAction, PyBackend};
use super::detect::{detect_python, PythonImpl, PythonInfo};

/// Driver CPython
pub struct PythonDriver<'a> {
    runner: &'a CommandRunner,
    python: PythonInfo,
}

impl<'a> PythonDriver<'a> {
    /* --------------------------------------------------------------------- */
    /* Construction                                                          */
    /* --------------------------------------------------------------------- */

    /// Initialise le driver (détection Python)
    pub fn new(runner: &'a CommandRunner) -> Result<Self, MuffinError> {
        let python = detect_python()?;
        Ok(Self { runner, python })
    }

    /// Accès aux infos Python détectées
    pub fn info(&self) -> &PythonInfo {
        &self.python
    }

    /* --------------------------------------------------------------------- */
    /* Exécution                                                             */
    /* --------------------------------------------------------------------- */

    /// Exécute une action Python décrite par `PyArgs`
    pub fn run(&self, args: &PyArgs) -> Result<(), MuffinError> {
        // Validation sémantique locale
        args.validate().map_err(MuffinError::ValidationFailed)?;

        // Vérification backend vs implémentation
        self.check_backend_compatibility(args)?;

        // Construction argv final
        let argv = args.to_argv();

        // Exécution via runner Muffin
        self.runner.run_with_env(
            args.python.as_os_str(),
            &argv,
            &args.env,
            args.root.as_deref(),
        )?;

        Ok(())
    }

    /* --------------------------------------------------------------------- */
    /* Raccourcis haut niveau                                                 */
    /* --------------------------------------------------------------------- */

    /// Compile du bytecode CPython (.pyc)
    pub fn compile_bytecode(
        &self,
        sources: &[PathBuf],
        root: Option<PathBuf>,
    ) -> Result<(), MuffinError> {
        let mut args = PyArgs::new();
        args.action = PyAction::CompileBytecode;

        for src in sources {
            args.add_source(src.clone());
        }

        if let Some(r) = root {
            args.set_root(r);
        }

        self.run(&args)
    }

    /// Build une extension native (setup.py)
    pub fn build_extension(
        &self,
        project_root: PathBuf,
        out: Option<PathBuf>,
    ) -> Result<(), MuffinError> {
        let mut args = PyArgs::new();
        args.action = PyAction::BuildExtension;
        args.set_root(project_root);

        if let Some(o) = out {
            args.set_output(o);
        }

        self.run(&args)
    }

    /// Génère un bundle zipapp
    pub fn bundle_zipapp(
        &self,
        root: PathBuf,
        entry: PathBuf,
        out: PathBuf,
    ) -> Result<(), MuffinError> {
        let mut args = PyArgs::new();
        args.action = PyAction::BundleZipapp;
        args.set_root(root);
        args.set_entry(entry);
        args.set_output(out);

        self.run(&args)
    }

    /// Génère un bundle natif Nuitka
    pub fn bundle_nuitka(
        &self,
        entry: PathBuf,
        out_dir: PathBuf,
    ) -> Result<(), MuffinError> {
        let mut args = PyArgs::new();
        args.backend = PyBackend::Nuitka;
        args.action = PyAction::BundleNuitka;
        args.set_entry(entry);
        args.set_output(out_dir);

        self.run(&args)
    }

    /* --------------------------------------------------------------------- */
    /* Validation interne                                                     */
    /* --------------------------------------------------------------------- */

    fn check_backend_compatibility(&self, args: &PyArgs) -> Result<(), MuffinError> {
        match args.backend {
            PyBackend::CPython => {
                if self.python.implementation != PythonImpl::CPython {
                    return Err(MuffinError::ValidationFailed(
                        "CPython backend requested but interpreter is not CPython".into(),
                    ));
                }
            }
            PyBackend::PyPy => {
                if self.python.implementation != PythonImpl::PyPy {
                    return Err(MuffinError::ValidationFailed(
                        "PyPy backend requested but interpreter is not PyPy".into(),
                    ));
                }
            }
            PyBackend::Nuitka => {
                // Nuitka nécessite CPython
                if self.python.implementation != PythonImpl::CPython {
                    return Err(MuffinError::ValidationFailed(
                        "Nuitka backend requires CPython interpreter".into(),
                    ));
                }
            }
        }

        Ok(())
    }
}
