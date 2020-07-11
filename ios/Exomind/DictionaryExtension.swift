//
//  DictionaryExtension.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-10-07.
//  Copyright © 2015 Exomind. All rights reserved.
//

import Foundation

// From http://stackoverflow.com/questions/24116271/whats-the-cleanest-way-of-applying-map-to-a-dictionary-in-swift

extension Dictionary {
    init(_ pairs: [Element]) {
        self.init()
        for (k, v) in pairs {
            self[k] = v
        }
    }

    func mapPairs<OutKey:Hashable, OutValue>(_ transform: (Element) throws -> (OutKey, OutValue)) rethrows -> [OutKey:OutValue] {
        return Dictionary<OutKey, OutValue>(try map(transform))
    }

    func filterPairs(_ includeElement: (Element) throws -> Bool) rethrows -> [Key:Value] {
        return Dictionary(try filter(includeElement))
    }
}

