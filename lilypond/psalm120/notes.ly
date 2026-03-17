melody = \relative c'' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g2 a4 f4 bes4 a4 g4 f4 e2 d2 r2 \break

  % Line 2
  d2 f4 g4 a4 bes4 a4 g2 fis4 g2 r2 \break

  % Line 3
  d2 d4 e4 f4 f4 g4 a4 bes2 g2 r2 \break

  % Line 4
  d2 c4 bes4 a4 g4 f4 g4 a2 g2 r2 \break

  % Line 5
  bes2 bes4 bes4 a4 f4 g4 a4 f2 d2 r2 \break

  % Line 6
  f2 g4 g4 a4 f4 g4 a4 bes2 a2 r2 \break

  % Line 7
  bes2 c4 d4 c4 bes4 a2 g2 f2 r2 \break

  % Line 8
  d2 f4 g4 a2 g2 g4 fis4 g1 \bar "|."
}
