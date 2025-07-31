/**
 * Utility functions for handling anchor link scrolling with header offset
 */

const HEADER_HEIGHT = 80; // h-20 = 5rem = 80px
const SCROLL_OFFSET = 20; // Additional offset for better visual spacing

/**
 * Scrolls to an element with proper header offset
 */
export function scrollToElement(element: HTMLElement, smooth = true) {
  const elementTop = element.offsetTop;
  const scrollTop = elementTop - HEADER_HEIGHT - SCROLL_OFFSET;

  window.scrollTo({
    top: Math.max(0, scrollTop),
    behavior: smooth ? "smooth" : "auto",
  });
}

/**
 * Handles anchor link clicks and scrolls to the target element
 */
export function handleAnchorClick(event: MouseEvent) {
  const target = event.target as HTMLElement;
  const link = target.closest('a[href^="#"]') as HTMLAnchorElement;

  if (!link) return;

  const href = link.getAttribute("href");
  if (!href || href === "#") return;

  const targetId = href.slice(1);
  const targetElement = document.getElementById(targetId);

  if (targetElement) {
    event.preventDefault();
    scrollToElement(targetElement);

    // Update URL without triggering scroll
    const url = new URL(window.location.href);
    url.hash = href;
    window.history.pushState({}, "", url.toString());
  }
}

/**
 * Handles initial page load with hash in URL
 */
export function handleInitialHash() {
  const hash = window.location.hash;
  if (!hash) return;

  const targetId = hash.slice(1);
  const targetElement = document.getElementById(targetId);

  if (targetElement) {
    // Use setTimeout to ensure the page is fully rendered
    setTimeout(() => {
      scrollToElement(targetElement, false);
    }, 100);
  }
}

/**
 * Sets up anchor link handling for the entire document
 */
export function setupAnchorHandling() {
  // Handle initial hash on page load
  handleInitialHash();

  // Handle hash changes (browser back/forward)
  window.addEventListener("hashchange", () => {
    handleInitialHash();
  });

  // Handle anchor link clicks
  document.addEventListener("click", handleAnchorClick);
}
