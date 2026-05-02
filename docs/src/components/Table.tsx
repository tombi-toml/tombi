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

    let rafId = 0;
    let needsLayoutSync = true;
    let resizeObserver: ResizeObserver | undefined;
    let clonedTable: HTMLTableElement | undefined;
    let clonedHead: HTMLTableSectionElement | undefined;

    const readCssLength = (name: string) =>
      Number.parseFloat(
        getComputedStyle(document.documentElement).getPropertyValue(name),
      ) || 0;

    const clearStickyHeader = () => {
      shellRef?.setAttribute("data-sticky-active", "false");
      stickyRef?.replaceChildren();
      clonedTable = undefined;
      clonedHead = undefined;
    };

    const ensureStickyHeader = () => {
      if (!shellRef || !scrollRef || !tableRef || !stickyRef) {
        return null;
      }

      const thead = tableRef.querySelector("thead");
      if (!thead) {
        clearStickyHeader();
        return null;
      }

      const originalCells = Array.from(thead.querySelectorAll("th"));
      if (originalCells.length === 0) {
        clearStickyHeader();
        return null;
      }

      if (!clonedTable || !clonedHead || !stickyRef.contains(clonedTable)) {
        stickyRef.replaceChildren();

        clonedTable = document.createElement("table");
        clonedTable.className = "mdx-table-cloned-head";
        clonedHead = thead.cloneNode(true) as HTMLTableSectionElement;
        clonedTable.appendChild(clonedHead);
        stickyRef.appendChild(clonedTable);
      }

      return { originalCells, clonedHead, clonedTable };
    };

    const updateStickyHeaderLayout = () => {
      if (!shellRef || !tableRef) {
        return;
      }

      const stickyHeader = ensureStickyHeader();
      if (!stickyHeader) {
        return;
      }

      const { originalCells, clonedHead, clonedTable } = stickyHeader;
      const tableRect = tableRef.getBoundingClientRect();

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
    };

    const updateStickyHeaderPosition = () => {
      if (!shellRef || !scrollRef) {
        return;
      }

      const stickyHeader = ensureStickyHeader();
      if (!stickyHeader) {
        return;
      }

      const { clonedHead, clonedTable } = stickyHeader;
      const stickyTop =
        readCssLength("--docs-fixed-header-height") +
        readCssLength("--docs-sticky-table-offset");

      const shellRect = shellRef.getBoundingClientRect();
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

    const renderStickyHeader = () => {
      if (needsLayoutSync) {
        updateStickyHeaderLayout();
        needsLayoutSync = false;
      }

      updateStickyHeaderPosition();
    };

    const requestRender = (syncLayout = false) => {
      needsLayoutSync = needsLayoutSync || syncLayout;
      cancelAnimationFrame(rafId);
      rafId = requestAnimationFrame(renderStickyHeader);
    };

    resizeObserver = new ResizeObserver(() => requestRender(true));
    resizeObserver.observe(shellRef);
    resizeObserver.observe(tableRef);

    const handleWindowScroll = () => requestRender();
    const handleWindowResize = () => requestRender(true);
    const handleTableScroll = () => requestRender();

    window.addEventListener("scroll", handleWindowScroll, { passive: true });
    window.addEventListener("resize", handleWindowResize);
    scrollRef.addEventListener("scroll", handleTableScroll, { passive: true });
    requestRender(true);

    onCleanup(() => {
      cancelAnimationFrame(rafId);
      resizeObserver?.disconnect();
      window.removeEventListener("scroll", handleWindowScroll);
      window.removeEventListener("resize", handleWindowResize);
      scrollRef?.removeEventListener("scroll", handleTableScroll);
      clearStickyHeader();
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
