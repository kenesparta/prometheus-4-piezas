// Diapositivas de apoyo (Typst) — espejo de ../slides.md, alineadas con
// ../guion-prometheus-4-piezas.md. La carne está en la DEMO; estas
// diapositivas son sólo anclas visuales, no para leer.
//
// Formato: 20 min de charla (01-15) + 10 min de demo en vivo (16-17), que se
// cierra con 18-19. Nada de navegador hasta el bloque de demo (los 02-04 solo
// muestran los QR, uno por página; el público entra desde su móvil, no el ponente).
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
#include "slides/02-qr-app.typ"
#include "slides/03-qr-prometheus.typ"
#include "slides/04-qr-grafana.typ"
#include "slides/05-madrugada.typ"
#include "slides/06-cuatro-piezas.typ"
#include "slides/07-pieza1-exporters.typ"
#include "slides/08-la-app.typ"
#include "slides/09-pieza2-servidor.typ"
#include "slides/10-cable.typ"
#include "slides/11-pieza3-promql.typ"
#include "slides/12-promql-app.typ"
#include "slides/13-pieza4-alertmanager.typ"
#include "slides/14-alertas-app.typ"
#include "slides/15-recap.typ"
#include "slides/16-demo-entra.typ"
#include "slides/17-demo-recorrido.typ"
#include "slides/18-para-seguir.typ"
#include "slides/19-cierre.typ"
