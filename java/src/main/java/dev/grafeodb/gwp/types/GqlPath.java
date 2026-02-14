package io.grafeodb.gwp.types;

import java.util.List;
import java.util.Optional;

/**
 * A path through a property graph: an alternating sequence of nodes and edges.
 *
 * <p>{@code nodes[i]} and {@code nodes[i+1]} are connected by {@code edges[i]}.</p>
 *
 * @param nodes the nodes in the path
 * @param edges the edges in the path
 */
public record GqlPath(List<GqlNode> nodes, List<GqlEdge> edges) {

    /**
     * The number of edges in the path (path length).
     *
     * @return the number of edges
     */
    public int length() {
        return edges.size();
    }

    /**
     * The start node of the path.
     *
     * @return the first node, or empty if the path has no nodes
     */
    public Optional<GqlNode> start() {
        return nodes.isEmpty() ? Optional.empty() : Optional.of(nodes.get(0));
    }

    /**
     * The end node of the path.
     *
     * @return the last node, or empty if the path has no nodes
     */
    public Optional<GqlNode> end() {
        return nodes.isEmpty() ? Optional.empty() : Optional.of(nodes.get(nodes.size() - 1));
    }
}
