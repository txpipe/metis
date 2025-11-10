interface Descriptor {
  mediaType: string;
  size: number;
  digest: string;
  annotations?: { [key: string]: string; };
}

interface OciManifest {
  schemaVersion: number;
  mediaType?: string;
  config: Descriptor;
  layers: Descriptor[];
  annotations?: { [key: string]: string; };
}
