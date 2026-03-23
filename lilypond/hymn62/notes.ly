melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  f4 f4 g2 f2 bes2 a2 \break

  % Line 2
  f4 g4 a4 c4 c2 bes2 c1 \break

  % Line 3
  c4 c4 d2 c2 bes4 a4 g2 a2 \break

  % Line 4
  f4 g4 a4 bes4 g2 f4 f1 \bar "|."
}
