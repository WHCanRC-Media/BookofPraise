melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  f4 bes4 f4 g4 g4 f4 ees4 d4 \break

  % Line 2
  d4 ees4 d4 c4 f4 f4 ees4 f4 \break

  % Line 3
  f4 bes4 c4 d4 bes4 ees4 d4 c4 \break

  % Line 4
  d4 bes4 g4 f4 bes4 bes4 a4 bes4 \bar "|."
}
