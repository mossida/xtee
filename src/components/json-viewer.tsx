"use client";

import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import { ChevronDown, ChevronRight } from "lucide-react";
import { useReducer } from "react";
import { flatMap, pipe } from "remeda";

const getAllPaths = (value: unknown, currentPath: string): string[] => {
  if (value === null || value === undefined || typeof value !== "object")
    return [];

  if (Array.isArray(value))
    return pipe(Array.from(value.entries()), (entries) => [
      currentPath,
      ...flatMap(entries, ([index, item]) =>
        getAllPaths(item, `${currentPath}-${index}`),
      ),
    ]);

  return pipe(Array.from(Object.entries(value)), (entries) => [
    currentPath,
    ...flatMap(entries, ([key, val]) =>
      getAllPaths(val, `${currentPath}-${key}`),
    ),
  ]);
};

type ExpandedState = Set<string>;
type ExpandedAction = { type: "TOGGLE"; path: string };

const expandedReducer = (
  state: ExpandedState,
  action: ExpandedAction,
): ExpandedState => {
  switch (action.type) {
    case "TOGGLE": {
      const newState = new Set(state);

      if (newState.has(action.path)) newState.delete(action.path);
      else newState.add(action.path);

      return newState;
    }
    default:
      return state;
  }
};

const getStableKey = (value: unknown, path: string): string => {
  if (value === null) return `${path}-null`;
  if (value === undefined) return `${path}-undefined`;
  if (typeof value === "object") return path;
  return `${path}-${String(value)}`;
};

const renderValue = (
  value: unknown,
  path: string,
  expandedPaths: Set<string>,
  onToggle: (path: string) => void,
  indent = 0,
) => {
  const isExpanded = expandedPaths.has(path);

  if (value === null)
    return <span className="text-muted-foreground">null</span>;
  if (value === undefined)
    return <span className="text-muted-foreground">undefined</span>;

  if (Array.isArray(value)) {
    if (value.length === 0)
      return <span className="text-muted-foreground">[]</span>;

    return (
      <div>
        <Button
          variant="ghost"
          size="sm"
          className="h-5 px-0.5 hover:bg-transparent hover:text-primary flex items-center"
          onClick={() => onToggle(path)}
        >
          {isExpanded ? (
            <ChevronDown className="h-3.5 w-3.5" />
          ) : (
            <ChevronRight className="h-3.5 w-3.5" />
          )}
          <span className="ml-0.5 text-muted-foreground hover:text-foreground">
            Array[{value.length}]
          </span>
        </Button>
        {isExpanded && (
          <div className="ml-4 border-l pl-3 border-border">
            {value.map((item, index) => {
              const itemPath = `${path}-${index}`;
              return (
                <div
                  key={getStableKey(item, itemPath)}
                  className="grid items-start py-[3px]"
                  style={{
                    gridTemplateColumns: "max-content max-content 1fr",
                    gap: "4px",
                  }}
                >
                  <span className="text-muted-foreground tabular-nums">
                    {index}
                  </span>
                  <span className="text-muted-foreground">:</span>
                  {renderValue(
                    item,
                    itemPath,
                    expandedPaths,
                    onToggle,
                    indent + 1,
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    );
  }

  if (typeof value === "object") {
    const entries = Object.entries(value);
    if (entries.length === 0)
      return <span className="text-muted-foreground">{}</span>;

    return (
      <div>
        <Button
          variant="ghost"
          size="sm"
          className="h-5 px-0.5 hover:bg-transparent hover:text-primary flex items-center"
          onClick={() => onToggle(path)}
        >
          {isExpanded ? (
            <ChevronDown className="h-3.5 w-3.5" />
          ) : (
            <ChevronRight className="h-3.5 w-3.5" />
          )}
          <span className="ml-0.5 text-muted-foreground hover:text-foreground">
            Object
          </span>
        </Button>
        {isExpanded && (
          <div className="ml-4 border-l pl-3 border-border">
            {entries.map(([key, val]) => {
              const valPath = `${path}-${key}`;
              return (
                <div
                  key={getStableKey(val, valPath)}
                  className="grid items-start py-[3px]"
                  style={{
                    gridTemplateColumns: "max-content max-content 1fr",
                    gap: "4px",
                  }}
                >
                  <span className="text-primary">{key}</span>
                  <span className="text-primary">:</span>
                  {renderValue(
                    val,
                    valPath,
                    expandedPaths,
                    onToggle,
                    indent + 1,
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    );
  }

  if (typeof value === "string")
    return <span className="text-green-600">"{value}"</span>;
  if (typeof value === "number")
    return <span className="text-orange-600">{value}</span>;
  if (typeof value === "boolean")
    return <span className="text-purple-600">{value.toString()}</span>;

  return <span>{String(value)}</span>;
};

type JsonViewerProps = {
  data: unknown;
  initialExpanded?: boolean;
  className?: string;
};

export const JsonViewer: React.FC<JsonViewerProps> = ({
  data,
  initialExpanded = false,
  className,
}) => {
  const [expandedPaths, dispatch] = useReducer(
    expandedReducer,
    new Set(initialExpanded ? getAllPaths(data, "root") : []),
  );

  const handleToggle = (path: string) => {
    dispatch({ type: "TOGGLE", path });
  };

  return (
    <Card className={cn("w-full overflow-auto", className)}>
      <CardContent className="p-3 font-mono text-sm">
        <div className="flex items-start">
          <div className="flex-1">
            {renderValue(data, "root", expandedPaths, handleToggle)}
          </div>
        </div>
      </CardContent>
    </Card>
  );
};
