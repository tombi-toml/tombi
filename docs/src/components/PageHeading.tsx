import { Meta, Title } from "@solidjs/meta";
import type { Component, JSX } from "solid-js";
import {
  DEFAULT_DESCRIPTION,
  DEFAULT_TITLE,
  DEFAULT_URL,
} from "~/remark/page-heading";

interface PageHeadingProps {
  title: string;
  description: string;
  url?: string;
  image?: string;
  children?: JSX.Element;
}

export const DEFAULT_IMAGE = `${import.meta.env.BASE_URL}/ogp.png`;

export const PageHeading: Component<PageHeadingProps> = (props) => {
  const title = () => {
    // For page title (browser tab)
    if (props.title) {
      return props.title;
    }
    if (typeof props.children === "string") {
      return props.children;
    }
    return DEFAULT_TITLE;
  };

  const description = () => {
    return props.description || DEFAULT_DESCRIPTION;
  };

  const url = props.url || DEFAULT_URL;
  const image = props.image || DEFAULT_IMAGE;

  return (
    <>
      <Title>{title()}</Title>
      <Meta name="description" content={description()} />

      {/* Open Graph / Facebook */}
      <Meta property="og:type" content="website" />
      <Meta property="og:url" content={url} />
      <Meta property="og:title" content={title()} />
      <Meta property="og:description" content={description()} />
      <Meta property="og:image" content={image} />
      <Meta property="og:site_name" content="Tombi" />

      {/* Twitter */}
      <Meta property="twitter:card" content="summary_large_image" />
      <Meta property="twitter:url" content={url} />
      <Meta property="twitter:title" content={title()} />
      <Meta property="twitter:description" content={description()} />
      <Meta property="twitter:image" content={image} />
    </>
  );
};
