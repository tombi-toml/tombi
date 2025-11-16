import { PageHeading } from "~/components/PageHeading";

export default function Playground() {
  return (
    <>
      <PageHeading
        title="Playground"
        description="Tombi's interactive playground."
        og_url={`${import.meta.env.BASE_URL}/playground`}
      />
      <div class="flex flex-col items-center justify-center min-h-[60vh]">
        <h1 class="text-4xl font-bold mb-4">🚧 Planned Feature 🚧</h1>
        <div class="text-xl text-gray-600 dark:text-gray-400 text-center">
          <p class="my-4">
            The Tombi Playground is planned as a future feature.
          </p>
          <p class="my-4">
            We aim to provide an interactive TOML formatting experience.
          </p>
        </div>
      </div>
    </>
  );
}
