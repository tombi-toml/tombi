import { Title } from "@solidjs/meta";
import { FaSolidFeather } from "solid-icons/fa";
import { TbBrandGithubFilled } from "solid-icons/tb";
import { FeatureCard } from "~/components/FeatureCard";
import { LinkButton } from "~/components/button/LinkButton";
import { Highlight } from "solid-highlight";

const FEATURES = [
  {
    emoji: "⚡️",
    title: "Fast",
    description: "High-performance formatter implemented in Rust",
  },
  {
    emoji: "🎯",
    title: "Accurate",
    description: "Full compliance with TOML specification",
  },
  {
    emoji: "🛤️",
    title: "Schema Driven",
    description: "Validate and format your TOML files using a JSON Schema",
  },
  {
    emoji: "🏬",
    title: "SchemaStore Integration",
    description:
      "Rich schema validation and completion through JSON Schema Store",
  },
  {
    emoji: "🚀",
    title: "Zero Configuration",
    description: "Start using powerful features instantly without any setup",
  },
  {
    emoji: "✨",
    title: "Magic Experience",
    description: "Magic tailing comma formatting, and magic trigger completion",
  },
] as const;

export default function Home() {
  return (
    <div>
      <Title>Tombi - Modern TOML Formatter</Title>

      <section class="text-center mb-24">
        <h1 class="sr-only">Tombi</h1>
        <div class="relative py-8 w-screen -mx-[calc((100vw-100%)/2)] overflow-hidden bg-gradient-to-b from-gray-900 to-gray-500">
          <img
            src={`${import.meta.env.BASE_URL}/tombi-transparent.svg`}
            alt="Tombi Logo"
            class="w-auto max-h-80 mx-auto px-8"
          />
        </div>

        <div class="mb-16">
          <p class="text-4xl text-tombi-primary dark:text-gray-300 mb-2 max-w-2xl mx-auto">
            Feature Rich TOML Toolkit
          </p>
          <p class="text-xl text-tombi-primary dark:text-gray-300 mb-2 max-w-2xl mx-auto">
            Bringing elegance and precision to your TOML configurations
          </p>
        </div>

        <div class="grid md:grid-cols-2 gap-x-16 gap-y-8 mb-16">
          {FEATURES.map((feature) => (
            <FeatureCard
              // @ts-ignore
              key={feature.title}
              emoji={feature.emoji}
              title={feature.title}
              description={feature.description}
            />
          ))}
        </div>

        <div class="flex gap-4 justify-center">
          <LinkButton
            href="/documentation/installation"
            variant="primary"
            class="text-xl"
          >
            <div class="flex items-center gap-2">
              Get Started <FaSolidFeather class="w-5 h-5" />
            </div>
          </LinkButton>
          <LinkButton
            href="https://github.com/tombi-toml/tombi"
            variant="secondary"
            class="text-xl"
          >
            <div class="flex items-center gap-2">
              Go to GitHub <TbBrandGithubFilled class="w-6 h-6" />
            </div>
          </LinkButton>
        </div>
      </section>

      <section class="max-w-3xl mx-auto px-4">
        <h2 class="text-3xl font-bold text-center mb-8 bg-gradient-to-r from-tombi-primary to-tombi-700 dark:from-white dark:to-tombi-200 bg-clip-text text-transparent">
          Simple and Easy to Use
        </h2>
        <Highlight language="toml">
          {`# Before
title="TOML Example"

[package]
name="my-project"
version="0.1.0"
authors=["John Doe <john@example.com>",]

# After
title = "TOML Example"

[package]
name = "my-project"
version = "0.1.0"
authors = [
  "John Doe <john@example.com>",
]`}
        </Highlight>
      </section>
    </div>
  );
}
