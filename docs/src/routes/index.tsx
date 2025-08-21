import { Title } from "@solidjs/meta";
import { FaSolidFeather } from "solid-icons/fa";
import { TbBrandGithubFilled } from "solid-icons/tb";
import { createSignal, onCleanup, onMount } from "solid-js";
import { LinkButton } from "~/components/button/LinkButton";
import { FeatureCard } from "~/components/FeatureCard";

const FEATURES = [
  {
    emoji: "⚡️",
    title: "Fast",
    description: "High-performance formatter implemented in Rust",
  },
  {
    emoji: "🎯",
    title: "Accurate",
    description: "Full compliance with TOML specification and toml-test",
  },
  {
    emoji: "🛤️",
    title: "Schema Driven",
    description: "Validate and format your TOML files using a JSON Schema",
  },
  {
    emoji: "🏬",
    title: "Schema Store",
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
    description:
      "Magic trailing comma formatting, and magic trigger completion",
  },
] as const;

export default function Home() {
  const [scrollY, setScrollY] = createSignal(0);

  onMount(() => {
    const handleScroll = () => {
      setScrollY(window.scrollY);
    };
    window.addEventListener("scroll", handleScroll);
    onCleanup(() => window.removeEventListener("scroll", handleScroll));
  });

  const getEagleStyle = () => {
    const rotation = Math.sin(scrollY() * 0.015) * 20;
    return {
      transform: `rotate(${rotation}deg)`,
      transition: "transform 0.1s ease-out",
    };
  };

  return (
    <div>
      <Title>Tombi - TOML Toolkit</Title>

      <section class="text-center mb-24">
        <h1 class="sr-only">Tombi</h1>
        <div class="relative py-8 w-screen -mx-[calc((100vw-100%)/2)] overflow-hidden bg-gradient-to-b from-gray-900 to-gray-500">
          <img
            src={`${import.meta.env.BASE_URL}/tombi-transparent.svg`}
            alt="Tombi Logo"
            class="w-auto max-h-80 mx-auto px-8"
          />
        </div>

        <div>
          <p class="text-4xl mb-4 max-w-2xl mx-auto">
            <span class="mr-6 inline-block" style={getEagleStyle()}>
              🦅
            </span>
            <span class="font-bold bg-gradient-to-r from-tombi-primary to-tombi-200 dark:from-white dark:to-tombi-200 bg-clip-text text-transparent">
              Feature-Rich TOML Toolkit
            </span>
            <span class="ml-6 inline-block" style={getEagleStyle()}>
              🦅
            </span>
          </p>
          <p class="text-xl text-tombi-primary dark:text-gray-400 mb-2 max-w-2xl mx-auto">
            Bringing elegance and precision to your TOML configurations
          </p>
        </div>

        <img
          src={`${import.meta.env.BASE_URL}/demo.gif`}
          alt="Tombi Demo"
          class="w-full my-16"
        />

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-x-8 gap-y-8 mb-16">
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
            href="/docs/installation"
            variant="primary"
            class="text-xl group"
          >
            <div class="flex items-center gap-2">
              Get Started{" "}
              <FaSolidFeather class="w-5 h-5 group-hover:animate-shake" />
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
    </div>
  );
}
