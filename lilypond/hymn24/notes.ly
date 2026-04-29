melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  f2 d4 c4 f2 g2 a4 a4 g2 \break

  % Line 2
  r4 a4 c4 bes4 a2 g2 f1 \break

  % Line 3
  c'2 a4 c4 d2 c2 bes4 a4 g2 \break

  % Line 4
  r4 c4 f,4 bes4 a2 g2 f1 \bar "|."
}
