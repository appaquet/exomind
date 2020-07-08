import SwiftUI

struct BootstrapView: View {
    @EnvironmentObject var appState: AppState

    var body: some View {
        NavigationView {
            VStack {
                MultilineTextView(text: self.$appState.config.bound)
                Spacer()
                HStack {
                    errorView()
                    Spacer()
                    Button("Save") {
                        self.appState.saveConfig()
                    }
                            .padding()
                }
            }
                    .navigationBarTitle("Bootstrap")
        }
    }

    func errorView() -> some View {
        appState.configError.map {
            Text("Error: \($0)").foregroundColor(.red)
        }
    }
}

// See: https://github.com/appcoda/MultiLineTextView/blob/master/SwiftUITextViewDemo/TextView.swift
struct MultilineTextView: UIViewRepresentable {
    @Binding var text: String

    func makeUIView(context: Context) -> UITextView {
        let view = UITextView()
        view.delegate = context.coordinator
        view.isScrollEnabled = true
        view.isEditable = true
        view.isUserInteractionEnabled = true
        view.autocorrectionType = .no
        view.autocapitalizationType = .none
        return view
    }

    func updateUIView(_ uiView: UITextView, context: Context) {
        uiView.text = text
    }

    func makeCoordinator() -> Coordinator {
        Coordinator($text)
    }

    class Coordinator: NSObject, UITextViewDelegate {
        var text: Binding<String>

        init(_ text: Binding<String>) {
            self.text = text
        }

        func textViewDidChange(_ textView: UITextView) {
            self.text.wrappedValue = textView.text
        }
    }
}

#if DEBUG
struct BootstrapView_Previews: PreviewProvider {
    static var previews: some View {
        BootstrapView()
    }
}
#endif