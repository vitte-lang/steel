# Ops

# Ops

Documentation “opérations” : exécution en CI, observabilité, caches, artefacts, sécurité, et procédures cross-platform.

---

## Index

- [CI](./ci.md)
- [Observabilité](./observability.md)
- [Cache & store](./cache.md)
- [Artefacts & répertoires](./artifacts.md)
- [Sandbox / capsule](./sandbox.md)
- [Reproductibilité](./reproducibility.md)
- [Dépannage](./troubleshooting.md)

---

## Rappels

- `build muffin` : **configure** → écrit `Muffinconfig.mff`
- `Muffin build` : **build** → lit `Muffinconfig.mff` → exécute le DAG

---

## Scope

Cette section couvre :

- intégration CI/CD (jobs, caches, matrices targets)
- logs, traces, timings, export JSON
- stratégies de cache (content-addressed vs mtime)
- sandboxing (capsule) et contraintes OS
- procédures de nettoyage/GC
- support Linux/macOS/Windows/BSD/Solaris