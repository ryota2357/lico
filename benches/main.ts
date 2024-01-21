import { path } from "./scripts/deps.ts";
import { hyperfine } from "./scripts/hyperfile.ts";
import {
  display_benchmark_result,
  dump_environment,
  load_benchmark_info,
} from "./scripts/util.ts";

await dump_environment([
  { name: "Date", command: "date +%Y/%m/%d %H:%M:%S %Z" },
  { name: "OS", command: "uname -om" },
  { name: "Lua", command: "lua -v" },
  { name: "Python", command: "python --version" },
]);

const info = await Promise.all([
  load_benchmark_info("fibonacci"),
  load_benchmark_info("mandelbrot"),
  load_benchmark_info("nbody"),
]);

await Promise.all(info.map(async (info) => {
  info.benches.sort(() => 0.5 - Math.random()); // shuffle

  const base_bench_index = info.benches.findIndex((bench) => bench.is_base);
  if (base_bench_index === -1) {
    console.error(`"${info.name}" not has base bench.`);
    Deno.exit(1);
  }

  const results = (await hyperfine({
    warmup: info.warmup,
    runs: info.runs,
    commands: info.benches.map((bench) => ({
      name: bench.name,
      command: `${bench.command} ${path.join(info.base_path, bench.path)}`,
    })),
  })).map((result, i) => ({
    name: info.benches[i].name,
    ...result,
  }));

  const base_result = results[base_bench_index];
  const compare_results = results.filter((_, i) => i !== base_bench_index);

  display_benchmark_result({
    name: info.name,
    base: { ...base_result },
    compare: compare_results.sort((a, b) => a.name.localeCompare(b.name)),
  });
}));
