

import SwiftUI
import Introspect

// TODO: Use stylesheet
// TODO: Closure for click callback

class CollectionPill {
    let id: String
    let name: String
    let icon: UIImage?
    let parent: CollectionPill?

    init(id: String, name: String, icon: UIImage? = nil, parent: CollectionPill? = nil) {
        self.id = id
        self.name = name
        self.icon = icon
        self.parent = parent
    }

    func finalIcon() -> UIImage {
        if let icon = self.icon {
            return icon
        } else {
            return ObjectsIcon.icon(forFontAwesome: .folderOpen, color: .black, dimension: 14)
        }
    }

    func hierarchyIcons() -> [UIImage] {
        var icons = [self.finalIcon()]
        var parent = self.parent
        while let curParent = parent {
            icons.append(curParent.finalIcon())
            parent = curParent.parent
        }
        return icons.reversed()
    }
}

struct CollectionPillView: View {
    private let collection: CollectionPill

    init(collection: CollectionPill) {
        self.collection = collection
    }

    var body: some View {
        HStack(spacing: 2) {
            self.iconsView()

            Text(self.collection.name)
                .font(.system(size: 12))
                .frame(maxWidth: 120, alignment: .leading)
                .fixedSize(horizontal: true, vertical: true) // text hugging
                .lineLimit(1)
        }
        .padding(5)
        .background(Color(UIColor("#d8d8d8")))
        .cornerRadius(12)
    }

    func iconsView() -> some View {
        let icons = self.collection.hierarchyIcons()
        return HStack(spacing: 2) {
            ForEach(icons, id: \.self) { icon in
                Image(uiImage: icon)
            }
        }
    }
}

struct CollectionPillsView: View {
    private let collections: [CollectionPill]

    init(collections: [CollectionPill]) {
        self.collections = collections

    }

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack {
                ForEach(self.collections, id: \.self.id) { collection in
                    CollectionPillView(collection: collection)
                }
            }
        }
        .introspectScrollView { scrollView in
            scrollView.bounces = false
        }
    }
}

struct CollectionPills_Previews: PreviewProvider {
    static var previews: some View {
        VStack {
            CollectionPillView(collection: CollectionPill(id: "col1", name: "Col1"))
            CollectionPillView(collection: CollectionPill(id: "col1", name: "Col2", icon: "😬".textToImage(ofSize: 12)))
            CollectionPillView(collection: CollectionPill(id: "col1", name: "Long long long long long text"))
            CollectionPillView(collection: CollectionPill(id: "col1", name: "Child", icon: "👶".textToImage(ofSize: 12), parent: CollectionPill(id: "col2", name: "Parent", icon: "🤷‍♂️".textToImage(ofSize: 12), parent: CollectionPill(id: "col3", name: "Grand parent", icon: "👴".textToImage(ofSize: 12)))))

            Spacer().frame(height: 50)

            CollectionPillsView(collections: [CollectionPill(id: "col1", name: "Col1"), CollectionPill(id: "col2", name: "Col2")])

            CollectionPillsView(collections: [
                CollectionPill(id: "col1", name: "Long text"),
                CollectionPill(id: "col2", name: "Long long long text", icon: "😬".textToImage(ofSize: 12)),
                CollectionPill(id: "col3", name: "Some child with hierarchy", icon: "👶".textToImage(ofSize: 12), parent: CollectionPill(id: "col2", name: "Parent", icon: "🤷‍♂️".textToImage(ofSize: 12), parent: CollectionPill(id: "col3", name: "Grand parent", icon: "👴".textToImage(ofSize: 12)))),
                CollectionPill(id: "col4", name: "Long long long text", icon: "😬".textToImage(ofSize: 12)),
            ])
        }
    }
}
