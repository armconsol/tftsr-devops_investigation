// Custom Edge for Query Builder Joins

import { memo } from 'react';
import {
  BaseEdge,
  EdgeLabelRenderer,
  getSmoothStepPath,
  type EdgeProps,
} from 'reactflow';

export const JoinEdge = memo((props: EdgeProps) => {
  const { sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition, data, id } = props;
  const [edgePath, labelX, labelY] = getSmoothStepPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  const joinType = data?.joinType || 'INNER';

  return (
    <>
      <BaseEdge id={id} path={edgePath} style={{ stroke: '#6366f1', strokeWidth: 2 }} />
      <EdgeLabelRenderer>
        <div
          style={{
            position: 'absolute',
            transform: `translate(-50%, -50%) translate(${labelX}px, ${labelY}px)`,
            background: '#fff',
            padding: '2px 8px',
            borderRadius: 4,
            fontSize: 11,
            fontWeight: 600,
            border: '1px solid #6366f1',
            color: '#6366f1',
            pointerEvents: 'all',
          }}
          className="nodrag nopan"
        >
          {joinType} JOIN
        </div>
      </EdgeLabelRenderer>
    </>
  );
});

JoinEdge.displayName = 'JoinEdge';
