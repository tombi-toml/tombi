import { TbBrandGithubFilled } from "solid-icons/tb";
import { LinkIconButton } from "../button/LinkIconButton";

export function HeaderSocialLinks(props: { mobile?: boolean }) {
  const linkClass = props.mobile
    ? "tombi-mobile-social-link flex items-center justify-center w-[2.8rem] h-[2.8rem]"
    : "hidden md:block w-6 h-6";

  return (
    <>
      <LinkIconButton
        href="https://x.com/tombi_toml"
        alt="X (Twitter)"
        class={linkClass}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 512 512"
          aria-label="X (Twitter)"
          width={props.mobile ? 28 : undefined}
          height={props.mobile ? 28 : undefined}
        >
          <title>X (Twitter)</title>
          <path
            fill="currentColor"
            d="M389.2 48h70.6L305.6 224.2 487 464H345L233.7 318.6 106.5 464H35.8L200.7 275.5 26.8 48H172.4L272.9 180.9 389.2 48zM364.4 421.8h39.1L151.1 88h-42L364.4 421.8z"
          />
        </svg>
      </LinkIconButton>
      <LinkIconButton
        href="https://github.com/tombi-toml/tombi"
        alt="GitHub"
        class={linkClass}
      >
        <TbBrandGithubFilled size={28} aria-label="GitHub" />
      </LinkIconButton>
    </>
  );
}
