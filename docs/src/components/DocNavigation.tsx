import { A, useLocation } from "@solidjs/router";
import { BiSolidLeftArrow, BiSolidRightArrow } from "solid-icons/bi";
import { flattenDocPages } from "~/utils/doc-index";
import docIndex from "../../doc-index.json";

export function DocNavigation() {
  const location = useLocation();
  const flatPages = flattenDocPages(docIndex);
  const currentIndex = () =>
    flatPages.findIndex(
      (page) => `${import.meta.env.BASE_URL}${page.path}` === location.pathname,
    );
  const nextPage = () =>
    currentIndex() === -1
      ? null
      : currentIndex() < flatPages.length - 1
        ? flatPages[currentIndex() + 1]
        : null;
  const prevPage = () =>
    currentIndex() === -1
      ? null
      : currentIndex() > 0
        ? flatPages[currentIndex() - 1]
        : null;

  return (
    <div class="mt-8 pt-8 border-t border-gray-200 flex justify-between">
      {(() => {
        const prev = prevPage();
        return (
          prev && (
            <A
              href={prev.path}
              class="no-underline text-blue-500 hover:text-blue-600"
            >
              <BiSolidLeftArrow size={12} /> {prev.title}
            </A>
          )
        );
      })()}
      {(() => {
        const next = nextPage();
        return (
          next && (
            <A
              href={next.path}
              class="no-underline text-blue-500 hover:text-blue-600 ml-auto"
            >
              {next.title} <BiSolidRightArrow size={12} />
            </A>
          )
        );
      })()}
    </div>
  );
}
