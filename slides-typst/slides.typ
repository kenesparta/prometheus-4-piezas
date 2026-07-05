// Diapositivas de apoyo (Typst) — espejo de ../slides.md, alineadas con
// ../guion-prometheus-4-piezas.md. La carne está en las DEMOS; estas
// diapositivas son sólo anclas visuales, no para leer.
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
#include "slides/05-pieza2-servidor.typ"
#include "slides/06-cable.typ"
#include "slides/07-pieza3-promql.typ"
#include "slides/08-pieza4-alertmanager.typ"
#include "slides/09-recap.typ"
#include "slides/10-para-seguir.typ"
#include "slides/11-cierre.typ"
