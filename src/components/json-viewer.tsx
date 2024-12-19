"use client";

import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { cn } from "@/lib/cn";
import { ChevronDown, ChevronRight } from "lucide-react";
import { useState } from "react";

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
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(new Set());

  const toggleExpand = (path: string) => {
    const newExpandedPaths = new Set(expandedPaths);
    if (newExpandedPaths.has(path)) {
      newExpandedPaths.delete(path);
    } else {
      newExpandedPaths.add(path);
    }
    setExpandedPaths(newExpandedPaths);
  };

  const renderValue = (value: unknown, path: string, indent = 0) => {
    if (value === null)
      return <span className="text-muted-foreground">null</span>;
    if (value === undefined)
      return <span className="text-muted-foreground">undefined</span>;

    if (Array.isArray(value)) {
      if (value.length === 0)
        return <span className="text-muted-foreground">[]</span>;

      const isExpanded = expandedPaths.has(path);
      return (
        <div>
          <Button
            variant="ghost"
            size="sm"
            className="h-6 px-1 hover:bg-transparent"
            onClick={() => toggleExpand(path)}
          >
            {isExpanded ? (
              <ChevronDown className="h-4 w-4" />
            ) : (
              <ChevronRight className="h-4 w-4" />
            )}
            <span className="ml-1">Array[{value.length}]</span>
          </Button>
          {isExpanded && (
            <div className="ml-4 border-l pl-3 border-border">
              {value.map((item, index) => (
                <div
                  key={`${path}-${
                    // biome-ignore lint/suspicious/noArrayIndexKey: safe
                    index
                  }`}
                  className="py-1"
                >
                  <span className="text-muted-foreground mr-2">{index}:</span>
                  {renderValue(item, `${path}-${index}`, indent + 1)}
                </div>
              ))}
            </div>
          )}
        </div>
      );
    }

    if (typeof value === "object") {
      const entries = Object.entries(value);
      if (entries.length === 0)
        return <span className="text-muted-foreground">{}</span>;

      const isExpanded = expandedPaths.has(path);
      return (
        <div>
          <Button
            variant="ghost"
            size="sm"
            className="h-6 px-1 hover:bg-transparent"
            onClick={() => toggleExpand(path)}
          >
            {isExpanded ? (
              <ChevronDown className="h-4 w-4" />
            ) : (
              <ChevronRight className="h-4 w-4" />
            )}
            <span className="ml-1">Object</span>
          </Button>
          {isExpanded && (
            <div className="ml-4 border-l pl-3 border-border">
              {entries.map(([key, val]) => (
                <div key={`${path}-${key}`} className="py-1">
                  <span className="text-primary mr-2">{key}:</span>
                  {renderValue(val, `${path}-${key}`, indent + 1)}
                </div>
              ))}
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

  return (
    <Card className={cn("w-full overflow-auto", className)}>
      <CardContent className="p-4 font-mono text-sm">
        {renderValue(data, "root")}
      </CardContent>
    </Card>
  );
};
