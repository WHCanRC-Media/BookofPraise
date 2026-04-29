melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'2 bes4 g4 f4 g4 bes4 a4 g2 \break

  % Line 2
  g2 bes4 bes4 a4 f4 g4 bes4 a2 \break

  % Line 3
  a2 c4 c4 c4 a4 bes4 a4 g2 \break

  % Line 4
  a2 bes4 g4 f4 g4 bes4 a4 g1 \bar "|."
}
