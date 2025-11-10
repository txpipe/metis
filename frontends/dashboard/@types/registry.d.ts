type Maybe<T> = T | null;
type InputMaybe<T> = Maybe<T>;
type Exact<T extends { [key: string]: unknown; }> = { [K in keyof T]: T[K] };
type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
type MakeEmpty<T extends { [key: string]: unknown; }, K extends keyof T> = { [_ in K]?: never };
type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
/** All built-in and custom scalars, mapped to their actual values */
interface Scalars {
  ID: { input: string; output: string; };
  String: { input: string; output: string; };
  Boolean: { input: boolean; output: boolean; };
  Int: { input: number; output: number; };
  Float: { input: number; output: number; };
  Time: { input: any; output: any; };
}

/**
 * Annotation is Key:Value pair representing custom data which is otherwise
 * not available in other fields.
 */
interface Annotation {
  /** Custom key */
  Key: Maybe<Scalars['String']['output']>;
  /** Value associated with the custom key */
  Value: Maybe<Scalars['String']['output']>;
}

/**
 * Contains various details about the CVE (Common Vulnerabilities and Exposures)
 * and a list of PackageInfo about the affected packages
 */
interface Cve {
  /** A detailed description of the CVE */
  Description: Maybe<Scalars['String']['output']>;
  /** CVE ID */
  Id: Maybe<Scalars['String']['output']>;
  /** Information on the packages in which the CVE was found */
  PackageList: Maybe<Array<Maybe<PackageInfo>>>;
  /** Reference for the given CVE */
  Reference: Maybe<Scalars['String']['output']>;
  /** The impact the CVE has, one of "UNKNOWN", "LOW", "MEDIUM", "HIGH", "CRITICAL" */
  Severity: Maybe<Scalars['String']['output']>;
  /** A short title describing the CVE */
  Title: Maybe<Scalars['String']['output']>;
}

/** Contains the diff results of subtracting Subtrahend's CVEs from Minuend's CVEs  */
interface CveDiffResult {
  /** List of CVE objects which are present in minuend but not in subtrahend */
  CVEList: Maybe<Array<Maybe<Cve>>>;
  /** Minuend is the image from which CVE's we subtract */
  Minuend: ImageIdentifier;
  /** The CVE pagination information, see PageInfo object for more details */
  Page: Maybe<PageInfo>;
  /** Subtrahend is the image which CVE's are subtracted */
  Subtrahend: ImageIdentifier;
  /** Summary of the findings for this image */
  Summary: Maybe<ImageVulnerabilitySummary>;
}

/** Contains the tag of the image and a list of CVEs */
interface CveResultForImage {
  /** List of CVE objects which affect this specific image:tag */
  CVEList: Maybe<Array<Maybe<Cve>>>;
  /** The CVE pagination information, see PageInfo object for more details */
  Page: Maybe<PageInfo>;
  /** Summary of the findings for this image */
  Summary: Maybe<ImageVulnerabilitySummary>;
  /** Tag affected by the CVEs */
  Tag: Maybe<Scalars['String']['output']>;
}

/**
 * Apply various types of filters to the queries made for repositories and images
 * For example we only want to display repositories which contain images with
 * a certain OS ar Architecture.
 */
interface Filter {
  /**
   * Only return images or repositories supporting the build architectures in the list
   * Should be values listed in the Go Language document https://go.dev/doc/install/source#environment
   */
  Arch: InputMaybe<Array<InputMaybe<Scalars['String']['input']>>>;
  /** Only return images or repositories with at least one signature */
  HasToBeSigned: InputMaybe<Scalars['Boolean']['input']>;
  /** Only returns images or repositories that are bookmarked or not bookmarked */
  IsBookmarked: InputMaybe<Scalars['Boolean']['input']>;
  /** Only returns images or repositories that are starred or not starred */
  IsStarred: InputMaybe<Scalars['Boolean']['input']>;
  /**
   * Only return images or repositories supporting the operating systems in the list
   * Should be values listed in the Go Language document https://go.dev/doc/install/source#environment
   */
  Os: InputMaybe<Array<InputMaybe<Scalars['String']['input']>>>;
}

/** Search results, can contain images, repositories and layers */
interface GlobalSearchResult {
  /** List of images matching the search criteria */
  Images: Maybe<Array<Maybe<ImageSummary>>>;
  /**
   * List of layers matching the search criteria
   * NOTE: the actual search logic for layers is not implemented at the moment
   */
  Layers: Maybe<Array<Maybe<LayerSummary>>>;
  /** Pagination information */
  Page: Maybe<PageInfo>;
  /** List of repositories matching the search criteria */
  Repos: Maybe<Array<Maybe<RepoSummary>>>;
}

/** Information on how a layer was created */
interface HistoryDescription {
  /** Author is the author of the build point. */
  Author: Maybe<Scalars['String']['output']>;
  /** Comment is a custom message set when creating the layer. */
  Comment: Maybe<Scalars['String']['output']>;
  /** Created is the time when the layer was created. */
  Created: Maybe<Scalars['Time']['output']>;
  /** CreatedBy is the command which created the layer. */
  CreatedBy: Maybe<Scalars['String']['output']>;
  /** EmptyLayer is used to mark if the history item created a filesystem diff. */
  EmptyLayer: Maybe<Scalars['Boolean']['output']>;
}

/** ImageIdentifier */
interface ImageIdentifier {
  /** The digest of the image */
  Digest: Maybe<Scalars['String']['output']>;
  /** The platform of the image */
  Platform: Maybe<Platform>;
  /** Repo name of the image */
  Repo: Scalars['String']['output'];
  /** The tag of the image */
  Tag: Scalars['String']['output'];
}

/** ImageInput  */
interface ImageInput {
  /** The digest of the image */
  Digest: InputMaybe<Scalars['String']['input']>;
  /** The platform of the image */
  Platform: InputMaybe<PlatformInput>;
  /** Repo name of the image */
  Repo: Scalars['String']['input'];
  /** The tag of the image */
  Tag: Scalars['String']['input'];
}

/**
 * Details about a specific image, it is used by queries returning a list of images
 * We define an image as a pairing or a repository and a tag belonging to that repository
 */
interface ImageSummary {
  /** Contact details of the people or organization responsible for the image */
  Authors: Maybe<Scalars['String']['output']>;
  /** Human-readable description of the software packaged in the image */
  Description: Maybe<Scalars['String']['output']>;
  /** The digest of the descriptor of this image */
  Digest: Maybe<Scalars['String']['output']>;
  /** URL to get documentation on the image */
  Documentation: Maybe<Scalars['String']['output']>;
  /** Number of downloads of the manifest of this image */
  DownloadCount: Maybe<Scalars['Int']['output']>;
  /** True if current user has delete permission on this tag. */
  IsDeletable: Maybe<Scalars['Boolean']['output']>;
  /** True if the image has a signature associated with it, false otherwise */
  IsSigned: Maybe<Scalars['Boolean']['output']>;
  /**
   * Labels associated with this image
   * NOTE: currently this field is unused
   */
  Labels: Maybe<Scalars['String']['output']>;
  /** Last time the image manifest was pulled */
  LastPullTimestamp: Maybe<Scalars['Time']['output']>;
  /** Timestamp of the last modification done to the image (from config or the last updated layer) */
  LastUpdated: Maybe<Scalars['Time']['output']>;
  /** License(s) under which contained software is distributed as an SPDX License Expression */
  Licenses: Maybe<Scalars['String']['output']>;
  /** List of manifests for all supported versions of the image for different operating systems and architectures */
  Manifests: Maybe<Array<Maybe<ManifestSummary>>>;
  /** The media type of the descriptor of this image */
  MediaType: Maybe<Scalars['String']['output']>;
  /** Timestamp when the image was pushed to the registry */
  PushTimestamp: Maybe<Scalars['Time']['output']>;
  /** Information about objects that reference this image */
  Referrers: Maybe<Array<Maybe<Referrer>>>;
  /** Name of the repository where the image is found */
  RepoName: Maybe<Scalars['String']['output']>;
  /** Info about signature validity */
  SignatureInfo: Maybe<Array<Maybe<SignatureSummary>>>;
  /** Total size of the files associated with all images (manifest, config, layers) */
  Size: Maybe<Scalars['String']['output']>;
  /** URL to get source code for building the image */
  Source: Maybe<Scalars['String']['output']>;
  /** Tag identifying the image within the repository */
  Tag: Maybe<Scalars['String']['output']>;
  /** Human-readable title of the image */
  Title: Maybe<Scalars['String']['output']>;
  /** Vendor associated with this image, the distributing entity, organization or individual */
  Vendor: Maybe<Scalars['String']['output']>;
  /** Short summary of the identified CVEs */
  Vulnerabilities: Maybe<ImageVulnerabilitySummary>;
}

/** Contains summary of vulnerabilities found in a specific image */
interface ImageVulnerabilitySummary {
  /** Count of all CVEs found in this image */
  Count: Maybe<Scalars['Int']['output']>;
  /** Coresponds to CVSS 3 score CRITICAL */
  CriticalCount: Maybe<Scalars['Int']['output']>;
  /** Coresponds to CVSS 3 score HIGH */
  HighCount: Maybe<Scalars['Int']['output']>;
  /** Coresponds to CVSS 3 score LOW */
  LowCount: Maybe<Scalars['Int']['output']>;
  /** Maximum severity of all CVEs found in this image */
  MaxSeverity: Maybe<Scalars['String']['output']>;
  /** Coresponds to CVSS 3 score MEDIUM */
  MediumCount: Maybe<Scalars['Int']['output']>;
  /** Coresponds to CVSS 3 score NONE */
  UnknownCount: Maybe<Scalars['Int']['output']>;
}

/** Information about how/when a layer was built */
interface LayerHistory {
  /** Additional information about how the layer was created. */
  HistoryDescription: Maybe<HistoryDescription>;
  /** Information specific to the layer such as size and digest. */
  Layer: Maybe<LayerSummary>;
}

/** Contains details about a specific layer which is part of an image */
interface LayerSummary {
  /** Digest of the layer content */
  Digest: Maybe<Scalars['String']['output']>;
  /** The size of the layer in bytes */
  Size: Maybe<Scalars['String']['output']>;
}

/** Details about a specific version of an image for a certain operating system and architecture. */
interface ManifestSummary {
  /** Value of the artifactType field if present else the value of the config media type */
  ArtifactType: Maybe<Scalars['String']['output']>;
  /** Digest of the config file associated with this image */
  ConfigDigest: Maybe<Scalars['String']['output']>;
  /** Digest of the manifest file associated with this image */
  Digest: Maybe<Scalars['String']['output']>;
  /** Total number of image manifest downloads from this repository */
  DownloadCount: Maybe<Scalars['Int']['output']>;
  /** Information about the history of the specific image, see LayerHistory */
  History: Maybe<Array<Maybe<LayerHistory>>>;
  /** True if the manifest has a signature associated with it, false otherwise */
  IsSigned: Maybe<Scalars['Boolean']['output']>;
  /** Timestamp of the last update to an image inside this repository */
  LastUpdated: Maybe<Scalars['Time']['output']>;
  /**
   * List of layers matching the search criteria
   * NOTE: the actual search logic for layers is not implemented at the moment
   */
  Layers: Maybe<Array<Maybe<LayerSummary>>>;
  /** OS and architecture supported by this image */
  Platform: Maybe<Platform>;
  /** Information about objects that reference this image */
  Referrers: Maybe<Array<Maybe<Referrer>>>;
  /** Info about signature validity */
  SignatureInfo: Maybe<Array<Maybe<SignatureSummary>>>;
  /** Total size of the files associated with this manifest (manifest, config, layers) */
  Size: Maybe<Scalars['String']['output']>;
  /** Short summary of the identified CVEs */
  Vulnerabilities: Maybe<ImageVulnerabilitySummary>;
}

/** Contains the name of the package, the current installed version and the version where the CVE was fixed */
interface PackageInfo {
  /** Minimum version of the package in which the CVE is fixed */
  FixedVersion: Maybe<Scalars['String']['output']>;
  /** Current version of the package, typically affected by the CVE */
  InstalledVersion: Maybe<Scalars['String']['output']>;
  /** Name of the package affected by a CVE */
  Name: Maybe<Scalars['String']['output']>;
  /** Path where the vulnerable package is located */
  PackagePath: Maybe<Scalars['String']['output']>;
}

/** Information on current page returned by the API */
interface PageInfo {
  /** The number of objects in this page */
  ItemCount: Scalars['Int']['output'];
  /** The total number of objects on all pages */
  TotalCount: Scalars['Int']['output'];
}

/**
 * Pagination parameters
 * If PageInput is empty, the request should return all objects.
 */
interface PageInput {
  /**
   * The maximum amount of results to return for this page
   * Negative values are not allowed
   */
  limit: InputMaybe<Scalars['Int']['input']>;
  /**
   * The results page number you want to receive
   * Negative values are not allowed
   */
  offset: InputMaybe<Scalars['Int']['input']>;
  /** The criteria used to sort the results on the page */
  sortBy: InputMaybe<SortCriteria>;
}

/** Paginated list of ImageSummary objects */
interface PaginatedImagesResult {
  /** Information on the returned page */
  Page: Maybe<PageInfo>;
  /** List of images */
  Results: Array<ImageSummary>;
}

/** Paginated list of RepoSummary objects */
interface PaginatedReposResult {
  /** Information on the returned page */
  Page: Maybe<PageInfo>;
  /** List of repositories */
  Results: Array<RepoSummary>;
}

/** Contains details about the OS and architecture of the image */
interface Platform {
  /**
   * The name of the compilation architecture which the image is built to run on,
   * Should be values listed in the Go Language document https://go.dev/doc/install/source#environment
   */
  Arch: Maybe<Scalars['String']['output']>;
  /**
   * The name of the operating system which the image is built to run on,
   * Should be values listed in the Go Language document https://go.dev/doc/install/source#environment
   */
  Os: Maybe<Scalars['String']['output']>;
}

/** PlatformInput contains the Os and the Arch of the input image */
interface PlatformInput {
  /** The arch of the image */
  Arch: InputMaybe<Scalars['String']['input']>;
  /** The os of the image */
  Os: InputMaybe<Scalars['String']['input']>;
}

/** Queries supported by the zot server */
interface Query {
  /** List of images on which the argument image depends on */
  BaseImageList: PaginatedImagesResult;
  /** Receive RepoSummaries of repos bookmarked by current user */
  BookmarkedRepos: PaginatedReposResult;
  /** Returns a list with CVE's that are present in `image` but not in `comparedImage` */
  CVEDiffListForImages: CveDiffResult;
  /** Returns a CVE list for the image specified in the argument */
  CVEListForImage: CveResultForImage;
  /** List of images which use the argument image */
  DerivedImageList: PaginatedImagesResult;
  /** Obtain detailed information about a repository and container images within */
  ExpandedRepoInfo: RepoInfo;
  /** Searches within repos, images, and layers */
  GlobalSearch: GlobalSearchResult;
  /** Search for a specific image using its name */
  Image: ImageSummary;
  /** Returns all the images from the specified repository | from all repositories if specified repository is "" */
  ImageList: PaginatedImagesResult;
  /** Returns a list of images vulnerable to the CVE of the specified ID */
  ImageListForCVE: PaginatedImagesResult;
  /** Returns a list of images which contain the specified digest  */
  ImageListForDigest: PaginatedImagesResult;
  /**
   * Returns a list of images that are no longer vulnerable to the CVE of the specified ID,
   * from the specified repository
   */
  ImageListWithCVEFixed: PaginatedImagesResult;
  /**
   * Returns a list of descriptors of an image or artifact manifest that are found in a <repo> and have a subject field of <digest>
   * Can be filtered based on a specific artifact type <type>
   */
  Referrers: Array<Maybe<Referrer>>;
  /** Returns a list of repositories with the newest tag (most recently created timestamp) */
  RepoListWithNewestImage: PaginatedReposResult;
  /** Receive RepoSummaries of repos starred by current user */
  StarredRepos: PaginatedReposResult;
}

/** Queries supported by the zot server */
interface QueryBaseImageListArgs {
  digest: InputMaybe<Scalars['String']['input']>;
  image: Scalars['String']['input'];
  requestedPage: InputMaybe<PageInput>;
}

/** Queries supported by the zot server */
interface QueryBookmarkedReposArgs {
  requestedPage: InputMaybe<PageInput>;
}

/** Queries supported by the zot server */
interface QueryCveDiffListForImagesArgs {
  excludedCVE: InputMaybe<Scalars['String']['input']>;
  minuend: ImageInput;
  requestedPage: InputMaybe<PageInput>;
  searchedCVE: InputMaybe<Scalars['String']['input']>;
  subtrahend: ImageInput;
}

/** Queries supported by the zot server */
interface QueryCveListForImageArgs {
  excludedCVE: InputMaybe<Scalars['String']['input']>;
  image: Scalars['String']['input'];
  requestedPage: InputMaybe<PageInput>;
  searchedCVE: InputMaybe<Scalars['String']['input']>;
  severity: InputMaybe<Scalars['String']['input']>;
}

/** Queries supported by the zot server */
interface QueryDerivedImageListArgs {
  digest: InputMaybe<Scalars['String']['input']>;
  image: Scalars['String']['input'];
  requestedPage: InputMaybe<PageInput>;
}

/** Queries supported by the zot server */
interface QueryExpandedRepoInfoArgs {
  repo: Scalars['String']['input'];
}

/** Queries supported by the zot server */
interface QueryGlobalSearchArgs {
  filter: InputMaybe<Filter>;
  query: Scalars['String']['input'];
  requestedPage: InputMaybe<PageInput>;
}

/** Queries supported by the zot server */
interface QueryImageArgs {
  image: Scalars['String']['input'];
}

/** Queries supported by the zot server */
interface QueryImageListArgs {
  repo: Scalars['String']['input'];
  requestedPage: InputMaybe<PageInput>;
}

/** Queries supported by the zot server */
interface QueryImageListForCveArgs {
  filter: InputMaybe<Filter>;
  id: Scalars['String']['input'];
  requestedPage: InputMaybe<PageInput>;
}

/** Queries supported by the zot server */
interface QueryImageListForDigestArgs {
  id: Scalars['String']['input'];
  requestedPage: InputMaybe<PageInput>;
}

/** Queries supported by the zot server */
interface QueryImageListWithCveFixedArgs {
  filter: InputMaybe<Filter>;
  id: Scalars['String']['input'];
  image: Scalars['String']['input'];
  requestedPage: InputMaybe<PageInput>;
}

/** Queries supported by the zot server */
interface QueryReferrersArgs {
  digest: Scalars['String']['input'];
  repo: Scalars['String']['input'];
  type: InputMaybe<Array<Scalars['String']['input']>>;
}

/** Queries supported by the zot server */
interface QueryRepoListWithNewestImageArgs {
  requestedPage: InputMaybe<PageInput>;
}

/** Queries supported by the zot server */
interface QueryStarredReposArgs {
  requestedPage: InputMaybe<PageInput>;
}

/** A referrer is an object which has a reference to a another object */
interface Referrer {
  /** A list of annotations associated with this referrer */
  Annotations: Array<Maybe<Annotation>>;
  /**
   * Referrer ArtifactType
   * See https://github.com/opencontainers/artifacts for more details
   */
  ArtifactType: Maybe<Scalars['String']['output']>;
  /** Digest of the manifest file of the referrer */
  Digest: Maybe<Scalars['String']['output']>;
  /**
   * Referrer MediaType
   * See https://github.com/opencontainers/artifacts for more details
   */
  MediaType: Maybe<Scalars['String']['output']>;
  /** Total size of the referrer files in bytes */
  Size: Maybe<Scalars['Int']['output']>;
}

/** Contains details about the repo: both general information on the repo, and the list of images */
interface RepoInfo {
  /** List of images in the repo */
  Images: Maybe<Array<Maybe<ImageSummary>>>;
  /** Details about the repository itself */
  Summary: Maybe<RepoSummary>;
}

/** Details of a specific repo, it is used by queries returning a list of repos */
interface RepoSummary {
  /** Total number of image manifest downloads from this repository */
  DownloadCount: Maybe<Scalars['Int']['output']>;
  /** True if the repository is bookmarked by the current user, false otherwise */
  IsBookmarked: Maybe<Scalars['Boolean']['output']>;
  /** True if the repository is starred by the current user, false otherwise */
  IsStarred: Maybe<Scalars['Boolean']['output']>;
  /** Timestamp of the last update to an image inside this repository */
  LastUpdated: Maybe<Scalars['Time']['output']>;
  /** Name of the repository */
  Name: Maybe<Scalars['String']['output']>;
  /**
   * Details of the newest image inside the repository
   * NOTE: not the image with the `latest` tag, the one with the most recent created timestamp
   */
  NewestImage: Maybe<ImageSummary>;
  /** List of platforms supported by this repository */
  Platforms: Maybe<Array<Maybe<Platform>>>;
  /** Rank represents how good the match was between the queried repo name and this repo summary. */
  Rank: Maybe<Scalars['Int']['output']>;
  /** Total size of the files within this repository */
  Size: Maybe<Scalars['String']['output']>;
  /** Number of stars attributed to this repository by users */
  StarCount: Maybe<Scalars['Int']['output']>;
  /** Vendors associated with this image, the distributing entities, organizations or individuals */
  Vendors: Maybe<Array<Maybe<Scalars['String']['output']>>>;
}

/** Contains details about the signature */
interface SignatureSummary {
  /** Author is the author of the signature */
  Author: Maybe<Scalars['String']['output']>;
  /** True if the signature is trusted, false otherwise */
  IsTrusted: Maybe<Scalars['Boolean']['output']>;
  /** Tool is the tool used for signing image */
  Tool: Maybe<Scalars['String']['output']>;
}

/**
 * All sort criteria usable with pagination, some of these criteria applies only
 * to certain queries. For example sort by severity is available for CVEs but not
 * for repositories
 */
type SortCriteria
  /**
   * Sort alphabetically ascending
   * Applies to: images, repositories and CVEs
   */
  = | 'ALPHABETIC_ASC'
  /**
   * Sort alphabetically descending
   * Applies to: images, repositories and CVEs
   */
    | 'ALPHABETIC_DSC'
  /**
   * Sort by the total download count
   * Applies to: repositories and images
   */
    | 'DOWNLOADS'
  /**
   * How relevant the result is based on the user input used while searching
   * Applies to: images and repositories
   */
    | 'RELEVANCE'
  /**
   * Sort from the most severe to the least severe
   * Applies to: CVEs
   */
    | 'SEVERITY'
  /**
   * Sort by the total number of stars given by users
   * Applies to: repositories
   */
    | 'STARS'
  /**
   * Sort by the most recently created timestamp of the images
   * Applies to: images and repositories
   */
    | 'UPDATE_TIME';
