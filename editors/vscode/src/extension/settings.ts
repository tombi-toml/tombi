export interface Settings {
  path?: string;
  args?: string[];
  env?: Record<string, string>;
  semanticTokens?: {
    strings?: { enabled?: boolean };
    comments?: { enabled?: boolean };
  };
}
