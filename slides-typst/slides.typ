// Diapositivas de apoyo (Typst) — espejo de ../slides.md, alineadas con
// ../guion-prometheus-4-piezas.md. La carne está en la DEMO; estas
// diapositivas son sólo anclas visuales, no para leer.
//
// Formato: 20 min de charla (01-12) + 10 min de demo en vivo (13-14), que se
// cierra con 15-16. Nada de navegador hasta el bloque de demo.
//
// ⚠️ Antes de cada charla: actualizar `url-app` en theme.typ (la IP pública de
// la app cambia con cada cluster).
//
// Compilar (las fuentes viven en fonts/; ver Makefile):
//   make            → slides.pdf
//   make watch      → recompila al guardar
// o a mano:
//   typst compile --font-path fonts slides.typ
//
// Estructura: theme.typ (paleta, tipografía y componentes) + slides/NN-*.typ
// (una diapositiva por archivo, en orden).

#import "theme.typ": setup
#show: setup

#include "slides/01-portada.typ"
#include "slides/02-madrugada.typ"
#include "slides/03-cuatro-piezas.typ"
#include "slides/04-pieza1-exporters.typ"
#include "slides/05-la-app.typ"
#include "slides/06-pieza2-servidor.typ"
#include "slides/07-cable.typ"
#include "slides/08-pieza3-promql.typ"
#include "slides/09-promql-app.typ"
#include "slides/10-pieza4-alertmanager.typ"
#include "slides/11-alertas-app.typ"
#include "slides/12-recap.typ"
#include "slides/13-demo-entra.typ"
#include "slides/14-demo-recorrido.typ"
#include "slides/15-para-seguir.typ"
#include "slides/16-cierre.typ"
