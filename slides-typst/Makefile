# Compila las diapositivas Typst de la charla.
#
#   make            → slides.pdf (descarga las fuentes si faltan)
#   make watch      → recompila al guardar (para editar)
#   make png        → PNG por diapositiva en build/png/ (revisión visual)
#   make fonts      → (re)descarga las fuentes a fonts/
#   make open       → compila y abre el PDF
#   make clean      → borra los artefactos generados (no toca fonts/)
#   make distclean  → clean + borra las fuentes descargadas

TYPST      ?= typst
FONT_DIR   := fonts
TYPST_ARGS := --font-path $(FONT_DIR)

MAIN := slides.typ
PDF  := slides.pdf
SRC  := $(MAIN) theme.typ $(wildcard slides/*.typ)

# Fuentes (OFL): Space Grotesk (texto) + JetBrains Mono (código).
SG_URL  := https://github.com/floriankarsten/space-grotesk/releases/download/2.0.0/SpaceGrotesk-2.0.0.zip
JBM_URL := https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip

# Un archivo "ancla" por familia: si está, se asume la familia completa.
SG_FONT  := $(FONT_DIR)/SpaceGrotesk-Regular.ttf
JBM_FONT := $(FONT_DIR)/JetBrainsMono-Regular.ttf
FONTS    := $(SG_FONT) $(JBM_FONT)

.DEFAULT_GOAL := pdf
.PHONY: pdf watch png open fonts clean distclean

pdf: $(PDF)

$(PDF): $(SRC) $(FONTS)
	$(TYPST) compile $(TYPST_ARGS) $(MAIN) $(PDF)

watch: $(FONTS)
	$(TYPST) watch $(TYPST_ARGS) $(MAIN) $(PDF)

png: $(SRC) $(FONTS)
	mkdir -p build/png
	rm -f build/png/*.png
	$(TYPST) compile $(TYPST_ARGS) --format png --ppi 144 $(MAIN) 'build/png/{0p}.png'

open: $(PDF)
	open $(PDF)

fonts: $(FONTS)

$(SG_FONT):
	@echo "→ Descargando Space Grotesk (OFL)…"
	@mkdir -p $(FONT_DIR)
	@tmp=$$(mktemp -d) && \
	curl -fsSL -o $$tmp/sg.zip "$(SG_URL)" && \
	unzip -q -o $$tmp/sg.zip -d $$tmp -x '__MACOSX/*' && \
	cp $$tmp/SpaceGrotesk-2.0.0/ttf/static/SpaceGrotesk-Regular.ttf \
	   $$tmp/SpaceGrotesk-2.0.0/ttf/static/SpaceGrotesk-Medium.ttf \
	   $$tmp/SpaceGrotesk-2.0.0/ttf/static/SpaceGrotesk-Bold.ttf \
	   $(FONT_DIR)/ && \
	cp $$tmp/SpaceGrotesk-2.0.0/OFL.txt $(FONT_DIR)/OFL-SpaceGrotesk.txt && \
	rm -rf $$tmp

$(JBM_FONT):
	@echo "→ Descargando JetBrains Mono (OFL)…"
	@mkdir -p $(FONT_DIR)
	@tmp=$$(mktemp -d) && \
	curl -fsSL -o $$tmp/jbm.zip "$(JBM_URL)" && \
	unzip -q -o $$tmp/jbm.zip -d $$tmp && \
	cp $$tmp/fonts/ttf/JetBrainsMono-Regular.ttf \
	   $$tmp/fonts/ttf/JetBrainsMono-Medium.ttf \
	   $$tmp/fonts/ttf/JetBrainsMono-Bold.ttf \
	   $(FONT_DIR)/ && \
	cp $$tmp/OFL.txt $(FONT_DIR)/OFL-JetBrainsMono.txt && \
	rm -rf $$tmp

clean:
	rm -f $(PDF)
	rm -rf build

distclean: clean
	rm -rf $(FONT_DIR)
