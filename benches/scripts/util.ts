import { colors, ensure, fs, is, path, sprintf, toml } from "./deps.ts";

const __dirname = path.dirname(path.fromFileUrl(import.meta.url));

export type BenchmarkResult = {
  readonly name: string;
  base: {
    name: string;
    mean: number;
    stddev: number;
    min: number;
    max: number;
  };
  compare: {
    name: string;
    mean: number;
    stddev: number;
    min: number;
    max: number;
  }[];
};

export function display_benchmark_result(data: BenchmarkResult) {
  const c = {
    title: (s: string) => colors.inverse(s),
    section: (s: string) => colors.bold(s),
    mean: (s: string) => colors.green(colors.bold(s)),
    stddev: (s: string) => colors.green(s),
    min: (s: string) => colors.cyan(s),
    max: (s: string) => colors.magenta(s),
  };
  const p = console.log;
  const f = (value: number, length: number) =>
    value.toString().substring(0, length);

  p();
  p(c.title(data.name));
  p(`  ${c.section(data.base.name)}`);
  p(`    Time:   ${
    sprintf(
      "%s  ±  %s",
      c.mean(`${f(data.base.mean * 1000, 7)} ms`),
      c.stddev(`${f(data.base.stddev * 1000, 7)} ms`),
    )
  }`);
  p(`    Range:  ${
    sprintf(
      "%s  …  %s",
      c.min(`${f(data.base.min * 1000, 7)} ms`),
      c.max(`${f(data.base.max * 1000, 7)} ms`),
    )
  }`);

  for (const compare of data.compare) {
    const mean_rate = colors.italic(
      c.mean(`×${f(compare.mean / data.base.mean, 4)}`),
    );

    p(`  ${c.section(compare.name)}`);
    p(`    Time:   ${
      sprintf(
        "%s  ±  %s",
        c.mean(`${f(compare.mean * 1000, 7)} ms`),
        c.stddev(`${f(compare.stddev * 1000, 7)} ms`),
      )
    }  (${mean_rate})`);
    p(`    Range:  ${
      sprintf(
        "%s  …  %s",
        c.min(`${f(compare.min * 1000, 7)} ms`),
        c.max(`${f(compare.max * 1000, 7)} ms`),
      )
    }`);
  }
}

export type BenchmarkInfo = {
  readonly name: string;
  readonly base_path: string;
  warmup: number;
  runs: number;
  benches: {
    name: string;
    is_base: boolean;
    command: string;
    path: string;
  }[];
};

export async function load_benchmark_info(
  name: string,
): Promise<BenchmarkInfo> {
  const base_path = path.join(__dirname, "..", "cases", name);
  if (await fs.exists(base_path, { isDirectory: true }) === false) {
    console.error(`"${name}" is not found.`);
    Deno.exit(1);
  }

  const toml_path = path.join(base_path, "info.toml");
  if (await fs.exists(toml_path, { isFile: true }) === false) {
    console.error(`"${name}" not has info.toml.`);
    Deno.exit(1);
  }

  const toml_text = await Deno.readTextFile(toml_path);
  const data = toml.parse(toml_text);

  const warmup = ensure(data["warmup"], is.Number);
  const runs = ensure(data["runs"], is.Number);
  const benches = ensure(
    data["benches"],
    is.ArrayOf(is.ObjectOf({
      name: is.String,
      base: is.OneOf([is.Boolean, is.Undefined]),
      command: is.String,
      path: is.String,
    })),
  );

  return {
    name,
    base_path,
    warmup,
    runs,
    benches: benches.map((bench) => ({
      ...bench,
      is_base: bench.base ?? false,
    })),
  };
}
