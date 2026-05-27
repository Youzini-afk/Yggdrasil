/**
 * Centralized icon re-exports.
 *
 * Phosphor v2 ships icons with the `Icon` suffix in component names. We
 * re-export with shorter, semantic names so component code reads naturally
 * (`<ArrowLeft />` instead of `<ArrowLeftIcon />`) and we keep tree-shaking
 * intact. Default weight is "regular"; surfaces that need a different weight
 * pass it explicitly.
 *
 * Add new icons here only when used in two or more places — single-use
 * imports stay local to their component.
 */

export {
  ArrowLeftIcon as ArrowLeft,
  ArrowRightIcon as ArrowRight,
  ArrowsClockwiseIcon as ArrowsClockwise,
  ArrowsLeftRightIcon as ArrowsLeftRight,
  ApertureIcon as Aperture,
  BellIcon as Bell,
  BookOpenIcon as BookOpen,
  BookOpenTextIcon as BookOpenText,
  BugBeetleIcon as Bug,
  CameraIcon as Camera,
  CaretDownIcon as CaretDown,
  ChatCircleIcon as ChatCircle,
  CheckCircleIcon as CheckCircle,
  CircleIcon as Circle,
  CloudIcon as Cloud,
  CopyIcon as Copy,
  DotsThreeIcon as DotsThree,
  DownloadIcon as Download,
  EyeIcon as Eye,
  EyeSlashIcon as EyeSlash,
  FolderIcon as Folder,
  FolderOpenIcon as FolderOpen,
  GearSixIcon as GearSix,
  GitBranchIcon as GitBranch,
  GithubLogoIcon as GithubLogo,
  GlobeIcon as Globe,
  InfoIcon as Info,
  KeyIcon as Key,
  LifebuoyIcon as Lifebuoy,
  LightningIcon as Lightning,
  LinkSimpleIcon as LinkSimple,
  ListBulletsIcon as ListBullets,
  MagnifyingGlassIcon as MagnifyingGlass,
  MoonIcon as Moon,
  NewspaperClippingIcon as Newspaper,
  PackageIcon as Package,
  PaperPlaneRightIcon as PaperPlane,
  PencilSimpleIcon as Pencil,
  PlayIcon as Play,
  PlugIcon as Plug,
  PlusIcon as Plus,
  QuestionIcon as Question,
  ShieldCheckIcon as ShieldCheck,
  SignpostIcon as Signpost,
  SignOutIcon as SignOut,
  StackIcon as Stack,
  StopCircleIcon as StopCircle,
  SunIcon as Sun,
  TerminalIcon as Terminal,
  UserIcon as User,
  WarningIcon as Warning,
  WrenchIcon as Wrench,
  XIcon as X,
  XCircleIcon as XCircle,
} from "@phosphor-icons/react";
