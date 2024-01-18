import { ensure, fs, is, jsonc, path, type PredicateType } from "./deps.ts";

const __dirname = path.dirname(path.fromFileUrl(import.meta.url));

type HyperfineCommandArgs = {
  warmup: number;
  runs: number;
  commands: {
    name: string;
    command: string;
  }[];
};

const isHyperfineResult = is.ObjectOf({
  command: is.String,
  mean: is.Number,
  stddev: is.Number,
  median: is.Number,
  user: is.Number,
  system: is.Number,
  min: is.Number,
  max: is.Number,
  times: is.ArrayOf(is.Number),
});

export type HyperfineResult = PredicateType<typeof isHyperfineResult>;

const textDecoder = new TextDecoder();

export async function hyperfine({
  warmup,
  runs,
  commands,
}: HyperfineCommandArgs): Promise<HyperfineResult[]> {
  const result_json = path.join(
    __dirname,
    `result-${crypto.randomUUID()}.json`,
  );
  await fs.ensureFile(result_json);

  const prosess = new Deno.Command(
    "hyperfine",
    {
      args: [
        `--warmup=${warmup}`,
        `--runs=${runs}`,
        `--export-json=${result_json}`,
        ...commands.flatMap((c) => [`--command-name=${c.name}`, c.command]),
      ],
      stdin: "piped",
      stdout: "piped",
      stderr: "piped",
    },
  ).spawn();

  const { code, stderr } = await prosess.output();
  if (code !== 0) {
    console.error(textDecoder.decode(stderr));
    await Deno.remove(result_json);
    Deno.exit(code);
  }

  const json = await Deno.readTextFile(result_json);
  const results = (jsonc.parse(json) as Record<string, unknown>)["results"];
  await Deno.remove(result_json);

  return ensure(results, is.ArrayOf(isHyperfineResult));
}
