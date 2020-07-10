
import Foundation
import SwiftProtobuf

public class QueryBuilder {
    private var inner: Exocore_Index_EntityQuery

    init() {
        self.inner = Exocore_Index_EntityQuery()
    }

    public static func withTrait<M: Message>(message: M? = nil) -> QueryBuilder {
        let builder = QueryBuilder()
        var traitPredicate = Exocore_Index_TraitPredicate()
        traitPredicate.traitName = M.protoMessageName
        builder.inner.trait = traitPredicate
        return builder
    }

    public static func withTrait(name: String) -> QueryBuilder {
        let builder = QueryBuilder()
        var traitPredicate = Exocore_Index_TraitPredicate()
        traitPredicate.traitName = name
        builder.inner.trait = traitPredicate
        return builder
    }

    public static func matching(query: String) -> QueryBuilder {
        let builder = QueryBuilder()
        var queryPredicate = Exocore_Index_MatchPredicate()
        queryPredicate.query = query
        builder.inner.match = queryPredicate
        return builder
    }

    public func count(count: Int) -> QueryBuilder {
        var paging = Exocore_Index_Paging()
        paging.count = UInt32(count)
        self.inner.paging = paging

        return self
    }

    public func build() -> Exocore_Index_EntityQuery {
        self.inner
    }
}
