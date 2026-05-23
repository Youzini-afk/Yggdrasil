import { cn } from "@/lib/cn";

export interface SkeletonProps extends React.HTMLAttributes<HTMLDivElement> {}

export function Skeleton({ className, ...props }: SkeletonProps) {
  return <div className={cn("rounded-[6px] shimmer", className)} {...props} />;
}

export function SkeletonText({
  lines = 3,
  className,
  widths,
}: {
  lines?: number;
  widths?: string[];
  className?: string;
}) {
  return (
    <div className={cn("flex flex-col gap-2", className)}>
      {Array.from({ length: lines }).map((_, i) => (
        <Skeleton key={i} className="h-3" style={{ width: widths?.[i] ?? `${100 - i * 6}%` }} />
      ))}
    </div>
  );
}
