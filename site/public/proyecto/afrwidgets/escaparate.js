// Selector de estética: cambia a la vez la mesa de muestra y la rejilla de los trece widgets.
const RUTA = "/proyecto/afrwidgets/w/";
const NOTAS = {
  suizo: "Suizo: blanco, negro y un rojo, con cifras de grotesca gruesa.",
  noche: "Noche: casi negro, cifras finas y un ámbar.",
  instrumento: "Instrumento: gris cálido, naranja y tipografía DIN.",
  aire: "Aire: sin panel, sólo tipografía fina sobre tu fondo.",
  grotesca: "Grotesca: grotesca gruesa, minúsculas finas y subrayado fosforito.",
  "grotesca-negra": "Grotesca negra: la misma grotesca, en blanco sobre tinta.",
};

const chips = [...document.querySelectorAll("[data-chips] button")];
const nota = document.querySelector("[data-nota]");
const mesa = document.querySelector("[data-mesa]");
const deMesa = [...document.querySelectorAll("[data-mesa] img")];
const deRejilla = [...document.querySelectorAll("[data-rejilla] img")];
let actual = "noche";

async function elegir(tema) {
  if (tema === actual) return;
  actual = tema;
  for (const c of chips) c.setAttribute("aria-pressed", String(c.dataset.tema === tema));
  nota.textContent = NOTAS[tema] ?? "";
  mesa.classList.add("cambiando");
  // La rejilla cambia ya (cada imagen aparece al cargar); la mesa espera a sus seis para no enseñar huecos.
  for (const img of deRejilla) img.src = `${RUTA}${img.dataset.k}-${tema}.webp`;
  await Promise.all(
    deMesa.map((img) => {
      const pre = new Image();
      pre.src = `${RUTA}${img.dataset.k}-${tema}.webp`;
      return pre.decode().catch(() => {});
    }),
  );
  if (tema !== actual) return;
  for (const img of deMesa) img.src = `${RUTA}${img.dataset.k}-${tema}.webp`;
  mesa.classList.remove("cambiando");
}

for (const c of chips) c.addEventListener("click", () => elegir(c.dataset.tema));
