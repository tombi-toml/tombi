import { A } from "@solidjs/router";

interface HeaderTabProps {
  href: string;
  children: string;
}

export function HeaderTab(props: HeaderTabProps) {
  return (
    <A
      href={props.href}
      class="header-section-tab text-white hover:text-white/80 text-lg font-medium no-underline p-2 btn-focus"
      activeClass="header-section-tab-active"
    >
      {props.children}
    </A>
  );
}
