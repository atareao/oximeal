# Git Flow — OxiMeal

Flujo de trabajo basado en **Git Flow** con *Conventional Commits* + **gitmoji**.

## Ramas

| Rama | Propósito | Origen | Destino de PR |
|---|---|---|---|
| `main` | Producción estable. | — | — |
| `development` | Integración de features. | `main` | `main` (release) |
| `feature/<nombre>` | Nuevas funcionalidades. | `development` | `development` |
| `hotfix/<nombre>` | Correcciones urgentes. | `main` | `main` (y luego `development`) |

> **Nunca** se hace commit directo a `main` ni a `development`: todo cambio entra por **Pull Request**.

## Flujo diario

```bash
git checkout development && git pull origin development
git checkout -b feature/<nombre>
# ... trabajo + tests en verde ...
git commit -m "✨ feat(domain): ..."
git push -u origin feature/<nombre>
# Abrir PR hacia development
```

## Hotfix

```bash
git checkout main && git pull origin main
git checkout -b hotfix/<nombre>
# ... fix + tests ...
git commit -m "🚑️ hotfix(api): ..."
git push -u origin hotfix/<nombre>
# PR hacia main; al fusionar, sincronizar development
```

## Formato de commit

```
<emoji> <tipo>(<scope>): <mensaje>
```

El `scope` es opcional. El tipo determina el *bump* de versión.

| tipo | emoji | bump |
|---|---|---|
| feat | ✨ | minor |
| fix | 🐛 | patch |
| hotfix | 🚑️ | patch |
| docs | 📝 | patch |
| refactor | ♻️ | patch |
| perf | ⚡ | patch |
| style | 💄 | patch |
| test | ✅ | patch |
| chore | 🔧 | patch |
| ci | 👷 | patch |
| revert | ⏪️ | patch |
| deps | ⬆️ | patch |
| breaking (`feat!` / `💥` / `BREAKING CHANGE`) | 💥 | major |

## Puertas de calidad antes de commit

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
