import { Meta, Title } from "@solidjs/meta";
import type { Component, JSX } from "solid-js";
import { DEFAULT_URL } from "~/remark/page-heading";

interface PageHeadingProps {
  title: string;
  description: string;
  og_url?: string;
  og_image?: string;
  children?: JSX.Element;
}

export const DEFAULT_IMAGE = `${import.meta.env.BASE_URL}/ogp.png`;

export const PageHeading: Component<PageHeadingProps> = (props) => {
  const title = () => {
    return props.title;
  };

  const description = () => {
    return props.description;
  };

  const og_url = props.og_url || DEFAULT_URL;
  const og_image = props.og_image || DEFAULT_IMAGE;

  return (
    <>
      <Title>{title()}</Title>
      <Meta name="description" content={description()} />

      {/* Open Graph / Facebook */}
      <Meta property="og:type" content="website" />
      <Meta property="og:url" content={og_url} />
      <Meta property="og:title" content={title()} />
      <Meta property="og:description" content={description()} />
      <Meta property="og:image" content={og_image} />
      <Meta property="og:site_name" content="Tombi" />

      {/* Twitter */}
      <Meta property="twitter:card" content="summary_large_image" />
      <Meta property="twitter:url" content={og_url} />
      <Meta property="twitter:title" content={title()} />
      <Meta property="twitter:description" content={description()} />
      <Meta property="twitter:image" content={og_image} />
    </>
  );
};
