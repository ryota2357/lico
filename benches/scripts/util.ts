import { colors, ensure, fs, is, path, printf, toml } from "./deps.ts";

const __dirname = path.dirname(path.fromFileUrl(import.meta.url));

export type BenchmarkResult = {
  readonly name: string;
  base: {
    name: string;
    mean: number;
    median: number;
  };
  compare: {
    name: string;
    mean: number;
    median: number;
  }[];
};

export function display_benchmark_result(data: BenchmarkResult) {
  printf("\n%s\n", colors.inverse(data.name));
  printf(" %s\n", data.base.name);
  printf("   mean:   %.8fms\n", data.base.mean * 1000);
  printf("   median: %.8fms\n", data.base.median * 1000);
  for (const compare of data.compare) {
    printf(" %s\n", compare.name);
    const mean = compare.mean / data.base.mean;
    const median = compare.median / data.base.median;
    printf("   mean:   ×%.8f (%.8fms)\n", mean, compare.mean * 1000);
    printf("   median: ×%.8f (%.8fms)\n", median, compare.median * 1000);
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
