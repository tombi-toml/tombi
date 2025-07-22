package tombi.configurations


import com.intellij.openapi.components.RoamingType
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.SimplePersistentStateComponent
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Storage
import com.intellij.openapi.components.service


/**
 * Persist settings known to this plugin to an application-level `tombi.xml`.
 * These settings are only shared among local projects.
 * 
 * There is at most one instance of this class at any given time,
 * serving as a thin wrapper for the "canonical" instance of [TombiConfigurations].
 * 
 * @see tombiConfigurations
 */
@State(name = "tombi", storages = [Storage("tombi.xml", roamingType = RoamingType.LOCAL)])
@Service(Service.Level.APP)
internal class TombiConfigurationService :
    SimplePersistentStateComponent<TombiConfigurations>(TombiConfigurations()) {
    
    companion object {
        fun getInstance() = service<TombiConfigurationService>()
    }
    
}


/**
 * Get a deep copy of the global [TombiConfigurations] instance.
 * 
 * @see TombiConfigurations
 */
internal val tombiConfigurations: TombiConfigurations
    get() = TombiConfigurationService.getInstance().state.copy()
