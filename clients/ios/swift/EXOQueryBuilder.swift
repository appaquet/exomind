
import Foundation
import SwiftProtobuf

public class EXOQueryBuilder {
    private var inner: Exocore_Index_EntityQuery

    init() {
        self.inner = Exocore_Index_EntityQuery()
    }

    public static func withTrait<M: Message>(message: M? = nil) -> EXOQueryBuilder {
        let builder = EXOQueryBuilder()
        var traitPredicate = Exocore_Index_TraitPredicate()
        traitPredicate.traitName = M.protoMessageName
        builder.inner.trait = traitPredicate
        return builder
    }

    public static func withTrait(name: String) -> EXOQueryBuilder {
        let builder = EXOQueryBuilder()
        var traitPredicate = Exocore_Index_TraitPredicate()
        traitPredicate.traitName = name
        builder.inner.trait = traitPredicate
        return builder
    }

    public static func matching(query: String) -> EXOQueryBuilder {
        let builder = EXOQueryBuilder()
        var queryPredicate = Exocore_Index_MatchPredicate()
        queryPredicate.query = query
        builder.inner.match = queryPredicate
        return builder
    }

    public func count(count: Int) -> EXOQueryBuilder {
        var paging = Exocore_Index_Paging()
        paging.count = UInt32(count)
        self.inner.paging = paging

        return self
    }

    public func build() -> Exocore_Index_EntityQuery {
        return self.inner
    }
}
