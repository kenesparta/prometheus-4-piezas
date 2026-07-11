# Si no lo mides, no lo controlas: Prometheus en 4 piezas

**Formato:** Ponencia Cloud Native · 30 min = **20 min de charla + 10 min de demo en vivo**
**Público:** Principiantes totales (sin experiencia previa con Prometheus)
**Demo:** corre sobre un cluster Kubernetes (GKE) y el público participa desde su móvil.
Mecánica detallada en `DEMO.md`; manifiestos en `k8s/`.

---

## Resumen

En un mundo de microservicios, contenedores efímeros y cargas de IA, saber qué pasa dentro de tus sistemas dejó de ser opcional. Prometheus es la herramienta graduada de la CNCF que respondió a esa necesidad y hoy es el punto de partida de cualquier stack Cloud Native. Esta charla desmitifica Prometheus reduciéndolo a sus cuatro piezas esenciales —exporters, el servidor de scraping, PromQL y Alertmanager— y demuestra que con esos cuatro conceptos ya entiendes el 80% de cómo funciona. Pasaremos de un sistema "ciego" a uno que se mide, se consulta y se alerta a sí mismo.

---

## La forma de la charla

Los primeros **20 minutos son charla pura**: las 4 piezas, en orden, sin abrir el
navegador. Los últimos **10 minutos son la demo**, y no es una demo de mirar: el
público entra a una app real desde su móvil, la usa, y las métricas que vemos en
pantalla son las que acaban de generar ellos.

Esa app es `pokeapi/` (Rust + Leptos, con MongoDB y Redis). Se presenta durante la
charla —justo después de la pieza 1, porque *ella misma es un exporter*— y se
convierte en el hilo conductor: las queries de la pieza 3 y las alertas de la
pieza 4 hablan de ella.

> **Por qué la demo va toda al final:** el público necesita generar tráfico durante
> varios minutos para que `rate(...[1m])` tenga algo que mostrar y las alertas
> lleguen a `Firing`. Repartir la demo entre las piezas dejaría cada momento sin
> datos. Además, un solo bloque es mucho más fácil de cronometrar y de salvar con
> capturas si la red falla.

---

## Antes de empezar: checklist técnico

Prepara esto la noche anterior y deja todo precargado para no perder tiempo en vivo. La mecánica completa está en `DEMO.md`.

- [ ] `kubectl config current-context` apunta al cluster (GKE) correcto.
- [ ] `./k8s/montar-charla.sh` ejecutado; `kubectl -n prometheus-demo get pods` todo **Running**.
- [ ] Desplegado **5-10 min antes** de la charla (las queries `rate(...[1m])` y la alerta necesitan datos).
- [ ] **La IP pública de la app**, anotada: `kubectl -n prometheus-demo get svc pokeapi-publico`.
- [ ] **URL de la app actualizada en las diapositivas** — cambia con cada cluster:
      `url-app` en `slides-typst/theme.typ` y la slide "Ahora, ustedes" de `slides.md`.
- [ ] `./k8s/port-forward.sh` corriendo en una terminal dedicada (abre 9090/9100/9093/3000 en localhost).
- [ ] Pestañas del navegador preabiertas: `localhost:9090` (Prometheus), `localhost:9100/metrics` (exporter), `localhost:3000/metrics` (la app), `localhost:9093` (Alertmanager).
- [ ] Terminal con fuente grande (18-20pt mínimo) y tema de alto contraste.
- [ ] Queries de PromQL copiadas en un bloc de notas, listas para pegar.
- [ ] **Plan B:** capturas de pantalla de cada acto por si la demo (o la red al cluster) falla.

> **Regla de oro de la demo:** nada que tarde más de 10 segundos en arrancar debería arrancarse en vivo. Todo desplegado y con port-forward antes de empezar; en vivo sólo se abren pestañas y se pegan queries.

---

## Estructura general

| # | Bloque | Tiempo | Diapositivas |
|---|--------|--------|--------------|
| 0 | Apertura: el problema | 2 min | 1-3 |
| 1 | Pieza 1 — Exporters | 3 min | 4 |
| 1b | La app que vamos a observar | 2 min | 5 |
| 2 | Pieza 2 — El servidor | 3 min | 6-7 |
| 3 | Pieza 3 — PromQL | 5 min | 8-9 |
| 4 | Pieza 4 — Alertmanager | 3 min | 10-11 |
| 5 | Recap | 2 min | 12 |
| | **Subtotal charla** | **20 min** | |
| 6 | **DEMO en vivo** (4 actos) | **9 min** | 13-14 |
| 7 | Cierre y preguntas | 1 min | 15-16 |
| | **Total** | **30 min** | |

> La pieza 3 (PromQL) se lleva el bloque más largo: es la de mayor valor. Si hay
> que recortar, se recorta de la 4, nunca de la 3.

---

## 0 · Apertura — El problema (2 min)

**Objetivo:** enganchar antes de explicar nada técnico.

**Guion hablado (idea, no leer literal):**

> "Imaginen las 2 de la madrugada. Su aplicación está corriendo, los servidores encendidos, todo 'verde'. Pero el dashboard está en blanco. No saben si la app está sana, si va a caer en cinco minutos, o si ya cayó. El sistema funciona... pero es invisible. Eso es lo que pasa cuando tienes infraestructura pero no tienes observabilidad."

**Puntos a clavar:**
- Monitoreo tradicional (¿está vivo el servidor?) ≠ observabilidad (¿qué está pasando *dentro*?).
- En Cloud Native los contenedores son efímeros: nacen y mueren en segundos. No puedes entrar a "mirar" cada uno.
- La CNCF lo resume así: la IA está subiendo el listón de la confiabilidad, y la pregunta dura es si puedes medir lo que corre en producción.
- Prometheus es la respuesta estándar: graduado en la CNCF, el segundo proyecto en lograrlo después de Kubernetes.

**Anunciar el trato:** "Veinte minutos de teoría, y en los últimos diez ustedes van a usar una app desde su celular mientras vemos, en vivo, sus propias métricas."

**Transición:** "Suena complejo, pero Prometheus en realidad son solo 4 piezas. Vamos una por una."

---

## 1 · Pieza 1 — Exporters: ¿de dónde salen las métricas? (3 min)

**Idea central:** Antes de medir algo, ese algo tiene que *exponer* sus números. Eso lo hace un exporter.

**Conceptos a explicar:**
- Un exporter expone métricas en un endpoint HTTP, normalmente `/metrics`, en texto plano.
- **Modelo pull, no push:** Prometheus *va a buscar* los datos; la app no los envía. Esto es contraintuitivo para principiantes — recalcarlo.
- Hay exporters para todo: bases de datos, el sistema operativo (`node_exporter`), apps propias instrumentadas con librerías.

**Gancho narrativo (opcional, 30s):** Dos proyectos de la CNCF pueden estar instalados perfectamente y aun así ser invisibles entre sí si nadie los "cablea". El servidor no adivina dónde están las métricas: hay que decírselo. Esa es la pieza que viene después.

**Prometer la demo:** "En diez minutos van a ver ese texto plano en pantalla — con los números que ustedes mismos habrán generado."

**Transición:** "Pero no quiero enseñarles un exporter de juguete. Quiero enseñarles una app de verdad."

---

## 1b · La app que vamos a observar (2 min)

**Idea central:** El exporter más interesante no es uno que instalas: es *tu propia app*, instrumentada.

**Qué contar sobre `pokeapi`:**
- Una pokédex web en **Rust + Leptos**. **MongoDB** guarda los usuarios; **Redis**, las sesiones y el caché.
- Buscas un pokémon: la primera vez sale de la **PokeAPI pública**; las siguientes, del **caché en Redis** — y la diferencia de latencia se ve en pantalla *y en las métricas*.
- Tiene login, registro y roles (`ADMIN` / `EDITOR` / `VISITOR`).
- Y **expone `/metrics`**, igual que el `node_exporter`. Es un exporter hecho en casa.

**Qué clavar:** "Esto es lo que van a hacer en el trabajo. No instalar un exporter: instrumentar su app. Y `/metrics` se ve exactamente igual."

> No dar todavía la URL: se entrega en la demo, para que el tráfico llegue
> concentrado y las alertas se enciendan cuando toca.

**Transición:** "Estos números están ahí, pero nadie los guarda ni los grafica. Necesitamos a quien los recolecte."

---

## 2 · Pieza 2 — El servidor de Prometheus: el recolector (3 min)

**Idea central:** El servidor de Prometheus hace *scrape*: visita cada endpoint `/metrics` cada cierto tiempo y guarda los valores como series temporales.

**Conceptos a explicar:**
- **Scrape:** Prometheus pregunta "¿cuánto vale esto ahora?" cada X segundos (típico: 15s).
- **Series temporales:** cada métrica se guarda con su marca de tiempo. Así puedes ver la evolución, no solo el valor actual.
- **Targets:** la lista de endpoints a scrapear. Esto es el "cableado" del que hablamos.
- Todo se configura en un archivo `prometheus.yml`.

**Mostrar el archivo de config (ya escrito, diapositiva 7):**

```yaml
# prometheus.yml (k8s/20-prometheus-config.yaml)
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']   # el exporter de la pieza 1

  - job_name: 'pokeapi'                   # nuestra app, un target más
    static_configs:
      - targets: ['pokeapi:3000']
```

> Señalar: "Aquí le decimos a Prometheus a dónde ir y cada cuánto. Ese es el cable que conecta los exporters con el recolector. Y fíjense: para Prometheus, nuestra app es un target igual que cualquier otro. No hay una categoría especial para 'apps propias'."

**Transición:** "Ya tenemos datos guardándose. Pero un montón de números no sirve de nada si no podemos hacerles preguntas. Ahí entra el lenguaje de Prometheus."

---

## 3 · Pieza 3 — PromQL: hacer preguntas a tus datos (5 min)

**Idea central:** PromQL es el lenguaje para consultar las métricas. Es el momento "ajá" de la charla: los números crudos se vuelven respuestas útiles.

**Concepto clave para principiantes:** No te asustes con la sintaxis. Vamos de lo simple a lo útil en 4 pasos.

### La escalera (diapositiva 8)

**1 — la más simple: el valor actual**

```promql
node_memory_MemAvailable_bytes
```

> "El valor crudo, igual que en el endpoint `/metrics`, pero ahora guardado en el tiempo."

**2 — filtrar con etiquetas**

```promql
node_cpu_seconds_total{mode="idle"}
```

> "Las etiquetas (lo que va entre llaves) te dejan filtrar. Aquí, solo el CPU en reposo."

**3 — `rate()`: la función estrella**

```promql
rate(node_cpu_seconds_total{mode="system"}[1m])
```

> "`rate()` calcula qué tan rápido crece algo. Esto es el corazón de PromQL: pasar de 'cuánto' a 'qué tan rápido'. Casi todo lo útil usa rate."

**4 — agregar varias series**

```promql
100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[1m])) * 100)
```

> "Y esto ya es una métrica de verdad: el porcentaje de uso de CPU. De números sueltos a un dashboard real."

**Qué clavar:** "Con `nombre`, `{etiquetas}`, `rate()` y una agregación, ya cubres la mayoría de lo que harás a diario."

### Las mismas 4 herramientas, sobre la app (diapositiva 9)

```promql
sum(rate(pokeapi_http_peticiones_total[1m])) by (rol)       # tráfico por rol
sum(rate(pokeapi_pokemon_consultas_total[5m])) by (origen)  # ¿caché o API pública?
pokeapi_sesiones_activas                                    # cuánta gente hay dentro
sum(increase(pokeapi_login_errores_total[1m]))              # logins fallidos
```

> Señalar: "Son los mismos cuatro ingredientes. Lo único que cambió es de qué estamos preguntando: ya no del CPU, sino del *negocio*. Cuántos usuarios, qué rol tienen, si el caché sirve de algo, si alguien está intentando entrar a la fuerza. Prometheus no distingue: para él todo son series de números."

**Transición:** "Pero nadie va a quedarse mirando estos gráficos las 24 horas. Queremos que el sistema nos avise solo. Última pieza."

---

## 4 · Pieza 4 — Alertmanager: que el sistema te avise (3 min)

**Idea central:** Una alerta es una query de PromQL con una condición. Cuando se cumple, Alertmanager se encarga de notificar.

**Conceptos a explicar:**
- En Prometheus defines **reglas de alerta**: "si esta query supera X durante Y tiempo, dispara."
- **Alertmanager** es una pieza separada que recibe las alertas y las enruta: email, Slack, PagerDuty, etc. También agrupa y silencia.
- Cierra el círculo: de mirar dashboards → a que el sistema hable solo.

**Mostrar una regla (diapositiva 10):**

```yaml
# alert.rules.yml (k8s/20-prometheus-config.yaml)
- alert: CPUAlto
  expr: 100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[1m])) * 100) > 80
  for: 1m
  labels:
    severity: warning
  annotations:
    summary: "Uso de CPU por encima del 80%"
```

> Señalar: "Fíjense que el `expr` es la misma query de PromQL de hace un minuto. Una alerta no es nada nuevo: es una pregunta con un umbral."

### Las alertas de la app (diapositiva 11)

| Alerta | Qué vigila |
|---|---|
| `PokeapiCaida` | la app no responde al scrape (target DOWN) |
| `PokeapiSinRedis` | perdió el caché y las sesiones |
| `PokeapiSinMongo` | perdió la base de usuarios |
| `PokeapiFuerzaBrutaDemo` | demasiados passwords incorrectos por minuto |

```yaml
- alert: PokeapiFuerzaBrutaDemo
  expr: >-
    sum(increase(pokeapi_login_errores_total{motivo="password_incorrecto"}[1m]))
    > 5
  for: 30s
```

**El remate:** "Esto es una alerta de **seguridad**, detección de fuerza bruta, y sigue siendo lo mismo: una pregunta, un umbral y un `for`. La app distingue internamente si el usuario no existe o si la contraseña está mal —eso va en la etiqueta `motivo`— pero nunca se lo dice a quien intenta entrar. Medir por dentro, callar por fuera."

**Transición:** "Ya están las 4 piezas. Ahora las vemos moverse."

---

## 5 · Recap (2 min) — diapositiva 12

1. **Exporters** → exponen las métricas.
2. **Servidor** → las recolecta (scrape) y las guarda.
3. **PromQL** → te deja preguntarles cosas.
4. **Alertmanager** → te avisa cuando algo va mal.

- Prometheus no vive solo: es la base sobre la que se apoyan **Grafana** (dashboards bonitos), **OpenTelemetry** (que se graduó en la CNCF y se está volviendo el estándar de instrumentación) y el resto del stack.
- "Lo básico es el 80%. Y ahora lo vamos a ver funcionando con datos que van a generar ustedes."

---

## 6 · DEMO en vivo — 4 actos (9 min)

> Mecánica exacta (qué teclear, qué señalar, qué hacer si falla) en `DEMO.md`.
> Aquí sólo el arco narrativo y el reparto del tiempo.

### Acto 0 · "Entren a la app" (~1 min) — diapositiva 13

Proyectar la URL y dejarla en pantalla. Mientras entran:

> "Entren, regístrense o usen `admin` / `123`. Busquen un pokémon. Busquen el mismo dos veces y miren la latencia. Y si quieren, equivóquense de contraseña a propósito — luego les enseño por qué."

**Clave:** no avanzar hasta que se oiga el ruido de gente tecleando. El tráfico
que generen aquí es el combustible de los actos 3 y 4.

### Acto 1 · Exporters, en vivo (~2 min)

`localhost:9100/metrics` → el `node_exporter`: texto plano, `nombre{etiquetas} valor`.

`localhost:3000/metrics` → **la app**. Buscar `pokeapi_http_peticiones_total` y
recargar: el número sube. "Cada uno de esos incrementos es un clic de alguien
aquí."

### Acto 2 · El servidor (~1 min)

`localhost:9090/targets` → `node`, `prometheus` y `pokeapi` en **UP**. "El cable
funciona. Prometheus ya está guardando lo que ustedes hacen."

### Acto 3 · PromQL (~3 min)

En `localhost:9090/graph`, pegar la escalera de la pieza 3 (las 4 del CPU, rápido)
y luego las de la app, despacio:

- `sum(rate(pokeapi_http_peticiones_total[1m])) by (rol)` — la curva sube con el público.
- `sum(rate(pokeapi_pokemon_consultas_total[5m])) by (origen)` — `cache` vs `api`: el caché gana.
- `pokeapi_sesiones_activas` — "esos son ustedes."

**El momento "ajá":** que vean su propio comportamiento colectivo dibujado en una
gráfica que se actualiza sola.

### Acto 4 · Alertmanager (~2 min)

`localhost:9090/alerts`:

- `CPUAlto` (umbral > 80%) en **Inactive** (verde): el sistema está sano.
- `CPUAltoDemo` (umbral > 1%, puesto bajo a propósito) en **Firing** (rojo).
- `PokeapiTraficoAltoDemo` en **Firing**: la encendió el público.
- Pedir a la sala: **"tecleen mal la contraseña, todos, durante un minuto"** →
  `PokeapiFuerzaBrutaDemo` pasa a **Pending** y luego a **Firing**.
- Cerrar en `localhost:9093`: la alerta llegó a Alertmanager. "Aquí es donde
  conectarías Slack, email o PagerDuty."

---

## 7 · Cierre (1 min) — diapositivas 15-16

**Recursos para seguir:**
- Documentación oficial: prometheus.io
- `node_exporter` y exporters de la comunidad.
- Grafana para visualización.
- Buscar "kube-prometheus-stack" cuando salten a Kubernetes.

**Cierre de una línea:**

> "Si no lo mides, no lo controlas. Ahora ya saben cómo medirlo."

---

## Notas de tiempo y recortes

- El reloj se controla en la **charla**, no en la demo: si a los 20 minutos no
  estás en la diapositiva 13, recorta sobre la marcha (la pieza 4 es lo primero
  que se comprime; PromQL, jamás).
- Dentro de la demo, el orden de sacrificio es: **acto 1** (basta con `/metrics`
  de la app, sáltate el `node_exporter`), luego el **acto 2** (`/targets` se ve en
  15 segundos). Los actos 3 y 4 son la razón de ser del bloque.
- Si el público **no genera tráfico** (sala tímida, wifi mala): tienes el
  generador de carga por terminal en `DEMO.md`. Arráncalo sin disimulo y
  cuéntalo — "hago trampa, pero la métrica no miente".
- Si **`PokeapiFuerzaBrutaDemo` no dispara**: con `increase(...[1m])` hace falta
  fallar el login de forma *sostenida* durante ~1 minuto, no un ráfaga de golpe.
  Insiste a la sala o usa el bucle de `DEMO.md`.
- **Preguntas:** al final, o sueltas durante la demo (es el momento en que la
  gente se anima). Las más comunes de principiantes: "¿pull vs push por qué?",
  "¿esto reemplaza a Grafana?" (no, se complementan), "¿funciona fuera de
  Kubernetes?" (sí).
