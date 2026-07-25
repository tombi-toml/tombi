package tombi.server


internal data class TombiCommand(
    val executable: String,
    val arguments: List<String> = emptyList(),
)
