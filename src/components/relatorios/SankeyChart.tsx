import { useEffect, useRef } from "react";
import * as echarts from "echarts";
import { formatBRL } from "@/lib/format";
import type { SankeyData } from "@/types/relatorios";

type ChartParam = {
  dataType?: "node" | "edge";
  name?: string;
  value?: number | string;
  data?: { source?: string; target?: string };
};

type Props = {
  data: SankeyData | null;
};

export function SankeyChart({ data }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<echarts.ECharts | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;
    chartRef.current = echarts.init(containerRef.current);
    return () => {
      chartRef.current?.dispose();
      chartRef.current = null;
    };
  }, []);

  useEffect(() => {
    if (!chartRef.current || !data) return;
    const formatParamValue = (value: ChartParam["value"]): string =>
      typeof value === "number" ? formatBRL(value) : String(value ?? "");
    chartRef.current.setOption({
      tooltip: {
        trigger: "item",
        triggerOn: "mousemove",
        formatter: (params: ChartParam | ChartParam[]) => {
          const p = Array.isArray(params) ? params[0] : params;
          const value = formatParamValue(p.value);
          if (p.dataType === "edge" && p.data) {
            return `${p.data.source} → ${p.data.target}: ${value}`;
          }
          return `${p.name}: ${value}`;
        },
      },
      series: [
        {
          type: "sankey",
          layout: "none",
          emphasis: { focus: "adjacency" },
          data: data.nodes,
          links: data.links,
          lineStyle: { color: "gradient", curveness: 0.5 },
          label: {
            fontSize: 12,
            formatter: (params: ChartParam) =>
              `${params.name}\n${formatParamValue(params.value)}`,
          },
          nodeWidth: 20,
          nodeGap: 12,
        },
      ],
    });
  }, [data]);

  useEffect(() => {
    const resize = () => chartRef.current?.resize();
    window.addEventListener("resize", resize);
    return () => window.removeEventListener("resize", resize);
  }, []);

  if (!data) {
    return (
      <div className="flex h-64 items-center justify-center rounded-lg border border-teller-border">
        <p className="text-sm text-teller-muted">
          Gráfico indisponível: resultado negativo no período
        </p>
      </div>
    );
  }

  return <div ref={containerRef} className="h-80 w-full" />;
}
