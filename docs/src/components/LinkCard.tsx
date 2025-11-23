import type { Component } from "solid-js";

interface LinkCardProps {
  title: string;
  description: string;
  image: string;
  url: string;
}

export const LinkCard: Component<LinkCardProps> = (props) => {
  if (props.image.startsWith("/")) {
    props.image = `${import.meta.env.BASE_URL}${props.image}`;
  }
  return (
    <div
      class="w-full my-6 border border-gray-200 dark:border-gray-700 rounded-lg bg-white dark:bg-gray-900 shadow-sm transition-all hover:translate-y-[-2px] hover:shadow-md"
      style="box-sizing: border-box; width: 100%; max-width: 100%; margin: 0 auto;"
    >
      <a
        href={props.url}
        class="block text-inherit no-underline p-4 rounded-lg outline-none focus-visible:ring-2 focus-visible:ring-tombi-primary focus-visible:ring-offset-2"
        target="_blank"
        rel="noopener noreferrer"
      >
        <div class="flex items-center gap-8 sm:(flex-col text-center gap-4)">
          <img
            src={props.image}
            alt={props.title}
            class="w-16 h-16 rounded flex-shrink-0 p-4"
            style="object-fit: cover"
          />
          <div class="flex-1">
            <h3 class="text-gray-800 dark:text-gray-100 text-xl m-0 mb-2">
              {props.title}
            </h3>
            <p class="text-gray-600 dark:text-gray-400 text-sm m-0 hidden sm:block">
              {props.description}
            </p>
          </div>
        </div>
      </a>
    </div>
  );
};
