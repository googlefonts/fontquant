import { run, get_parametric, version } from "fontquant-web";

const versionEl = document.getElementById("version") as HTMLParagraphElement;
const fileInput = document.getElementById("file") as HTMLInputElement;
const locationInput = document.getElementById("location") as HTMLInputElement;
const parametricInput = document.getElementById(
  "parametric",
) as HTMLInputElement;
const statusEl = document.getElementById("status") as HTMLDivElement;
const outputEl = document.getElementById("output") as HTMLPreElement;

versionEl.textContent = `fontquant-web v${version()}`;

async function analyze(file: File): Promise<void> {
  statusEl.textContent = `Reading ${file.name}...`;
  statusEl.className = "";
  outputEl.textContent = "";

  try {
    const bytes = new Uint8Array(await file.arrayBuffer());
    const location = locationInput.value.trim() || undefined;
    const useParametric = parametricInput.checked;

    statusEl.textContent = `Running ${useParametric ? "parametric" : "all"} quantifiers...`;

    const started = performance.now();
    const json = useParametric
      ? get_parametric(bytes, location)
      : run(bytes, location);
    const elapsed = performance.now() - started;

    const parsed: unknown = JSON.parse(json);
    statusEl.textContent = `${file.name} inspected in ${elapsed.toFixed(1)} ms`;
    outputEl.textContent = JSON.stringify(parsed, null, 2);
  } catch (err) {
    statusEl.className = "error";
    statusEl.textContent = String(err);
    outputEl.textContent = "";
  }
}

function rerun(): void {
  const file = fileInput.files?.[0];
  if (file) {
    analyze(file);
  }
}

fileInput.addEventListener("change", rerun);
locationInput.addEventListener("change", rerun);
parametricInput.addEventListener("change", rerun);
