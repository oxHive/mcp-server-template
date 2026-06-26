_default:
  @just --choose

build:
  cargo build

test:
  cargo test

install:
  cargo install --path . --force

release-patch:
  just _release patch

release-minor:
  just _release minor

release-major:
  just _release major

_release bump:
  cargo release --execute {{{{bump}}}}
{% if include_dashboard %}

[working-directory: '{{dashboard_dir}}']
setup-frontend:
  bun create vue@latest .

[working-directory: '{{dashboard_dir}}']
dashboard:
  bun run build
{% endif %}
