//! Muffin OCaml – Driver (MAX)
//!
//! Driver d’exécution OCaml pour Muffin.
//!
//! Responsabilités :
//! - détecter l’environnement OCaml
//! - vérifier la compatibilité backend / capacités
//! - construire la commande finale
//! - déléguer l’exécution au CommandRunner
//!
//! Hors périmètre :
//! - graphe
//! - planification
//! - cache CAS

use super::error::MuffinError;
use super::runner::process::CommandRunner;

use super::args::{OcamlArgs, OcamlBackend};
use super::detect::{detect_ocaml, OcamlInfo};

/// Driver OCaml Muffin
pub struct OcamlDriver<'a> {
    runner: &'a CommandRunner,
    info: OcamlInfo,
}

impl<'a> OcamlDriver<'a> {
    /* --------------------------------------------------------------------- */
    /* Construction                                                          */
    /* --------------------------------------------------------------------- */

    /// Initialise le driver (détection OCaml)
    pub fn new(runner: &'a CommandRunner) -> Result<Self, MuffinError> {
        let info = detect_ocaml()?;
        Ok(Self { runner, info })
    }

    /// Accès aux informations détectées
    pub fn info(&self) -> &OcamlInfo {
        &self.info
    }

    /* --------------------------------------------------------------------- */
    /* Exécution                                                             */
    /* --------------------------------------------------------------------- */

    /// Exécute une compilation OCaml décrite par `OcamlArgs`
    pub fn run(&self, args: &OcamlArgs) -> Result<(), MuffinError> {
        // Validation locale
        args.validate().map_err(MuffinError::ValidationFailed)?;

        // Vérification des capacités
        self.check_backend_support(args)?;

        // Sélection du compilateur
        let compiler = match args.backend {
            OcamlBackend::Ocamlc => self
                .info
                .ocamlc
                .as_ref()
                .ok_or_else(|| {
                    MuffinError::ValidationFailed(
                        "ocamlc requested but not available".into(),
                    )
                })?,
            OcamlBackend::Ocamlopt => self
                .info
                .ocamlopt
                .as_ref()
                .ok_or_else(|| {
                    MuffinError::ValidationFailed(
                        "ocamlopt requested but not available".into(),
                    )
                })?,
        };

        // Génération argv final
        let argv = args.to_argv();

        // Exécution via runner Muffin
        self.runner.run(
            compiler.as_os_str(),
            &argv,
            None,
        )?;

        Ok(())
    }

    /* --------------------------------------------------------------------- */
    /* Raccourcis haut niveau                                                 */
    /* --------------------------------------------------------------------- */

    /// Compile un exécutable OCaml natif simple
    pub fn compile_executable(&self, args: &OcamlArgs) -> Result<(), MuffinError> {
        self.run(args)
    }

    /// Compile une librairie OCaml
    pub fn compile_library(&self, args: &OcamlArgs) -> Result<(), MuffinError> {
        self.run(args)
    }

    /* --------------------------------------------------------------------- */
    /* Validation interne                                                     */
    /* --------------------------------------------------------------------- */

    fn check_backend_support(&self, args: &OcamlArgs) -> Result<(), MuffinError> {
        match args.backend {
            OcamlBackend::Ocamlc => {
                if !self.info.has_bytecode {
                    return Err(MuffinError::ValidationFailed(
                        "OCaml bytecode backend (ocamlc) not supported".into(),
                    ));
                }
            }
            OcamlBackend::Ocamlopt => {
                if !self.info.has_native {
                    return Err(MuffinError::ValidationFailed(
                        "OCaml native backend (ocamlopt) not supported".into(),
                    ));
                }
            }
        }
        Ok(())
    }
}
