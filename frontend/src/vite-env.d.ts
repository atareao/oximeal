/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_VALET_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
