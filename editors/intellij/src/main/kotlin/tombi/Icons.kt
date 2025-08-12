package tombi

import com.intellij.openapi.util.IconLoader


private inline fun <reified H> H.loadIcon(path: String) =
    IconLoader.getIcon(path, H::class.java)


@Suppress("ObjectPropertyName")
internal object Icons {
    val _16 by lazy { loadIcon("icons/16.svg") }
}
