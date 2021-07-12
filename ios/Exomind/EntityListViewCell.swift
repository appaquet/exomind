

import SwiftUI


class EntityListViewCellHost: UITableViewCell {
    var host: UIHostingController<AnyView>?
//
//    init(parent: UIViewController) {
//        self.inner = UIHostingController(rootView: EntityListViewCell())
//
//        super.init(style: .default, reuseIdentifier: "cell")
//
//        inner.view.translatesAutoresizingMaskIntoConstraints = false
//        inner.view.frame = self.contentView.bounds
//        inner.didMove(toParent: parent) // TODO: This cylce
//
//        self.contentView.addSubview(inner.view)
//    }
//
//    required init?(coder: NSCoder) {
//      fatalError("init(coder:) has not been implemented")
//  }
}


struct EntityListViewCell: View {
    var body: some View {
        VStack {
            Text(/*@START_MENU_TOKEN@*/"Hello, World!"/*@END_MENU_TOKEN@*/)
            Spacer().frame(height: CGFloat(Int.random(in: 10...200)))
            Text("Comment ca va?")
        }
    }
}

struct EntityListViewCell_Previews: PreviewProvider {
    static var previews: some View {
        EntityListViewCell()
    }
}
