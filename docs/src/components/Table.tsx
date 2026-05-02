import { onCleanup, onMount } from "solid-js";
import type { JSX } from "solid-js";

export function Table(props: JSX.TableHTMLAttributes<HTMLTableElement>) {
  const { children, ...tableProps } = props;
  let shellRef: HTMLDivElement | undefined;
  let scrollRef: HTMLDivElement | undefined;
  let tableRef: HTMLTableElement | undefined;
  let stickyRef: HTMLDivElement | undefined;

  onMount(() => {
    if (!shellRef || !scrollRef || !tableRef || !stickyRef) {
      return;
    }

    const stickyTop = 80;
    let rafId = 0;
    let resizeObserver: ResizeObserver | undefined;

    const renderStickyHeader = () => {
      if (!shellRef || !scrollRef || !tableRef || !stickyRef) {
        return;
      }

      const thead = tableRef.querySelector("thead");
      if (!thead) {
        shellRef.dataset.stickyActive = "false";
        stickyRef.replaceChildren();
        return;
      }

      const shellRect = shellRef.getBoundingClientRect();
      const tableRect = tableRef.getBoundingClientRect();
      const originalCells = Array.from(thead.querySelectorAll("th"));
      if (originalCells.length === 0) {
        shellRef.dataset.stickyActive = "false";
        stickyRef.replaceChildren();
        return;
      }

      stickyRef.replaceChildren();

      const clonedTable = document.createElement("table");
      clonedTable.className = "mdx-table-cloned-head";
      const clonedHead = thead.cloneNode(true) as HTMLTableSectionElement;
      clonedTable.appendChild(clonedHead);
      stickyRef.appendChild(clonedTable);

      const clonedCells = Array.from(clonedHead.querySelectorAll("th"));
      originalCells.forEach((cell, index) => {
        const clonedCell = clonedCells[index];
        if (!clonedCell) {
          return;
        }
        const width = cell.getBoundingClientRect().width;
        clonedCell.style.width = `${width}px`;
        clonedCell.style.minWidth = `${width}px`;
        clonedCell.style.maxWidth = `${width}px`;
      });

      clonedTable.style.width = `${tableRect.width}px`;

      const stickyHeight = clonedHead.getBoundingClientRect().height;
      const shouldStick =
        shellRect.top <= stickyTop && shellRect.bottom > stickyTop + stickyHeight;

      shellRef.dataset.stickyActive = shouldStick ? "true" : "false";
      if (!shouldStick) {
        return;
      }

      stickyRef.style.top = `${stickyTop}px`;
      stickyRef.style.left = `${shellRect.left}px`;
      stickyRef.style.width = `${shellRect.width}px`;
      clonedTable.style.transform = `translateX(${-scrollRef.scrollLeft}px)`;
    };

    const requestRender = () => {
      cancelAnimationFrame(rafId);
      rafId = requestAnimationFrame(renderStickyHeader);
    };

    resizeObserver = new ResizeObserver(requestRender);
    resizeObserver.observe(shellRef);
    resizeObserver.observe(tableRef);

    window.addEventListener("scroll", requestRender, { passive: true });
    window.addEventListener("resize", requestRender);
    scrollRef.addEventListener("scroll", requestRender, { passive: true });
    requestRender();

    onCleanup(() => {
      cancelAnimationFrame(rafId);
      resizeObserver?.disconnect();
      window.removeEventListener("scroll", requestRender);
      window.removeEventListener("resize", requestRender);
      scrollRef?.removeEventListener("scroll", requestRender);
    });
  });

  return (
    <div class="mdx-table-shell" data-sticky-active="false" ref={shellRef}>
      <div class="mdx-table-sticky-head" aria-hidden="true" ref={stickyRef} />
      <div class="mdx-table-scroll" ref={scrollRef}>
        <table {...tableProps} ref={tableRef}>
          {children}
        </table>
      </div>
    </div>
  );
}
