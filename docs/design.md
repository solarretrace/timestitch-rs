
# Timestitch design

There are two use-cases supported by this tool:

1. "Journal mode": Constructing formatted overviews of data by period, sourced from a collection of implicitely per-day files (i.e., files whose naming scheme encodes the timeline position of the data contained within.)
2. "Timeline mode": Constructing formatted overvies of data by period, sourced from a collection of explicitly annotated date-time data fields without any particulare naming scheme.

To support both of these, we want to organize the inputs and process them into a uniform intermediate representation. From this representation, we can generate the desired output.

The following steps are an outline of how things should behave:

1. Process files
	+ File names are processed by NameMatcher
	+ File contents are processed by either deserialization or FileMatcher if deserialization fails.
	+ Matcher parameters are provided in the Prefs file.
	+ The result is a list of entries.
2. Process entries
	+ Entries are sorted and wrapped for rendering.
	+ Sort keys are provided by the Prefs file.
3. Write entries
	+ Output is written by table-gen-rs.




# Time measurement

1. Each event is associated with a time interval.
2. Time intervals must implement Cell (PartialOrd + Display)
3. Time intervals are generic over the Calendar.
4. Calendar may be real (via Chrono) or arbitrary (via Regex matching)
5. Calendar times must be resolved to provide ordering, which is done against a list of entries.

