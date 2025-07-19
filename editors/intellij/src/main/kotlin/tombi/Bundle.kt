package tombi

import com.intellij.DynamicBundle
import org.jetbrains.annotations.Nls
import org.jetbrains.annotations.PropertyKey


private const val BUNDLE_PATH = "messages.tombi"


private object Bundle {
    val instance = DynamicBundle(Bundle::class.java, BUNDLE_PATH)
}


/**
 * Retrieve a message from `messages/tombi.properties`.
 */
@Nls
internal fun message(
    @PropertyKey(resourceBundle = BUNDLE_PATH) key: String,
    vararg params: Any
) =
    Bundle.instance.getMessage(key, *params)
