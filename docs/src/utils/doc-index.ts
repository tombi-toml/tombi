export type DicIndex = {
  title: string;
  description: string;
  path: string;
  children?: DicIndex[];
};

export type FlattenedDocPage = {
  title: string;
  description: string;
  path: string;
};

export function flattenDocPages(pages: DicIndex[]): FlattenedDocPage[] {
  return pages.reduce<FlattenedDocPage[]>((acc, page) => {
    acc.push(page);
    if (page.children) {
      acc.push(...flattenDocPages(page.children));
    }
    return acc;
  }, []);
}
