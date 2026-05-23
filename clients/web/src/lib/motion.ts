/**
 * Centralized Motion spring presets so timing stays consistent across
 * primitives. `useReducedMotion` is honored automatically because
 * `<MotionConfig reducedMotion="user">` wraps the app — Motion zeroes
 * these out for users who request reduced motion.
 *
 * Use the named preset instead of inline `{ stiffness, damping }`:
 *
 *   import { SPRING } from "@/lib/motion";
 *   <motion.div transition={SPRING.modal} />
 */

export const SPRING = {
  /** Snappy: dropdowns, toasts, small affordances. */
  snap: { type: "spring" as const, stiffness: 520, damping: 38 },
  /** Soft: cards, list rows, larger surfaces. */
  soft: { type: "spring" as const, stiffness: 320, damping: 32 },
  /** Modal: dialog scale-in, slightly more deliberate. */
  modal: { type: "spring" as const, stiffness: 360, damping: 34 },
};

/** Brief eased duration for cross-fade / step transitions. */
export const FADE = {
  short: { duration: 0.18 },
  natural: { duration: 0.24 },
};
