# Diapositivas en Typst

Versión en [Typst](https://typst.app/) de las diapositivas de la charla, espejo
diapositiva a diapositiva de `../slides.md` (Marp) y alineada con
`../guion-prometheus-4-piezas.md`. Como en el deck original: la carne está en las
demos; esto son anclas visuales, no diapositivas para leer.

## Estructura

```
slides-typst/
├── Makefile            # compila, descarga fuentes, genera PNGs
├── slides.typ          # punto de entrada: aplica el tema e incluye las diapositivas
├── theme.typ           # paleta, tipografía y componentes (slide, pieza, demo-strip…)
├── slides/             # una diapositiva por archivo, en orden
│   ├── 01-portada.typ
│   ├── 02-madrugada.typ
│   ├── ...
│   └── 11-cierre.typ
├── fonts/              # tipografías vendorizadas (descargadas por `make fonts`)
└── slides.pdf          # artefacto generado (make)
```

## Compilar

Requiere **Typst** (probado con 0.15): `brew install typst`. No usa paquetes de
Typst: con las fuentes ya descargadas compila offline.

```bash
make            # slides.pdf (descarga las fuentes a fonts/ si faltan)
make watch      # recompila al guardar (para editar)
make png        # un PNG por diapositiva en build/png/ (revisión visual)
make fonts      # (re)descarga las fuentes
make open       # compila y abre el PDF
make clean      # borra artefactos generados (no toca fonts/)
make distclean  # clean + borra las fuentes
```

A mano, sin make: `typst compile --font-path fonts slides.typ` (el `--font-path`
importa: ahí viven las fuentes del deck).

## Tipografía

- **Space Grotesk** (texto y títulos) y **JetBrains Mono** (código), ambas con
  licencia OFL; los `.ttf` y sus licencias viven en `fonts/`.
- `make fonts` las descarga de sus releases oficiales (GitHub), así que el
  directorio es regenerable; también se pueden dejar versionadas para compilar
  sin red.
- Si faltan, Typst cae a fuentes del sistema (Helvetica Neue/Menlo): compila
  igual, pero con otro aspecto.
- Space Grotesk no tiene cursiva: el énfasis `_..._` se renderiza en peso medio
  y color ámbar (definido en `theme.typ`).

## Presentar

- Abrir `slides.pdf` a pantalla completa (Vista Previa con ⌘⇧F, o Skim).
- Las demos van fuera del PDF (navegador y terminal): la mecánica está en
  `../DEMO.md` y los manifiestos en `../k8s/`.

## Mantenimiento

Este deck duplica el contenido de `slides.md`. Cualquier cambio en una query,
un puerto o un umbral de alerta debe mantenerse consistente en: el guion,
`DEMO.md`, `slides.md`, **estas diapositivas** (`slides/NN-*.typ`) y los
manifiestos de `k8s/` (ver `CLAUDE.md` en la raíz).

El contenido vive en `slides/` (un archivo por diapositiva); el aspecto vive en
`theme.typ` (paleta arriba, componentes abajo; el naranja `#E6522C` es el de la
marca Prometheus).
