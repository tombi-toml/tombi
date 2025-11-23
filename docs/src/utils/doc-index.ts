export type DocIndex = {
  title: string;
  description: string;
  path: string;
  children?: DocIndex[];
};

export type FlattenedDocPage = {
  title: string;
  description: string;
  path: string;
};

export function flattenDocPages(pages: DocIndex[]): FlattenedDocPage[] {
  return pages.reduce<FlattenedDocPage[]>((acc, page) => {
    acc.push(page);
    if (page.children) {
      acc.push(...flattenDocPages(page.children));
    }
    return acc;
  }, []);
}
