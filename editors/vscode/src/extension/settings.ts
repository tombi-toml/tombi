export interface Settings {
  path?: string;
  args?: string[];
  env?: Record<string, string>;
  semanticTokens?: {
    strings?: { enable?: boolean };
    comments?: { enable?: boolean };
  };
}
